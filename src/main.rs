#![feature(file_create_new)]

use std::{
    collections::HashSet,
    fs::{self, File},
    path::Path,
};

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use embedded_graphics::prelude::Size;
use indicatif::ParallelProgressIterator;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use scraper::Html;
use serde::Serialize;
use serde_json::json;
use tinybmp::{Bpp, RawBmp};
use url::Url;
use uuid::Uuid;

const BASE_URL: &str = "https://forum.swissmicros.com";
const OUT_DIR: &str = "out";

const BMP_TYPES: &[&str] = &["image/bmp", "image/x-ms-bmp"];

macro_rules! selector {
    ($raw:expr) => {{
        static SELECTOR: once_cell::sync::OnceCell<scraper::Selector> =
            once_cell::sync::OnceCell::new();
        SELECTOR.get_or_init(|| scraper::Selector::parse($raw).unwrap())
    }};
}

fn main() -> Result<()> {
    let base: Url = Url::parse(BASE_URL)?;
    if !Path::new(OUT_DIR).exists() {
        fs::create_dir(OUT_DIR)?;
    }

    let mut topics = HashSet::new();
    for page in 0.. {
        let url = format!("{BASE_URL}/viewforum.php?f=14&start={}", 25 * page);
        let res = ureq::get(&url).call()?;
        if res.status() != 200 {
            bail!("Failed to fetch page #{page}: {}", res.status());
        }

        let dom = Html::parse_document(&res.into_string()?);
        let posts = dom
            .select(selector!(".topictitle"))
            .map(|x| x.value().attr("href").unwrap()[2..].to_owned())
            .collect::<Vec<_>>();
        let count = posts.len();
        topics.extend(posts);

        if count != 25 {
            break;
        }
    }

    println!("[*] Found {} posts", topics.len());
    let images = topics
        .par_iter()
        .progress_count(topics.len() as u64)
        .flat_map(|post| {
            let mut images = Vec::new();
            let mut seen = HashSet::new();

            'outer: for page in 0.. {
                let url = format!("{BASE_URL}/{post}&start={}", page * 10);
                let post_id = post
                    .split("t=")
                    .nth(1)
                    .unwrap()
                    .split('&')
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap();
                let res = ureq::get(&url).call().unwrap();
                if res.status() != 200 {
                    panic!("Failed to fetch post {}: {}", post, res.status());
                }

                let dom = Html::parse_document(&res.into_string().unwrap());
                let posts = dom.select(selector!(".post"));
                let start_length = seen.len();

                for post in posts {
                    let id = post.value().attr("id").unwrap();
                    if !seen.insert(id.to_owned()) {
                        break 'outer;
                    }

                    let date = post.select(selector!("time")).next().unwrap();
                    let date = date.value().attr("datetime").unwrap();
                    let date = DateTime::parse_from_rfc3339(date)
                        .unwrap()
                        .with_timezone(&Utc);

                    for image in post.select(selector!("img.postimage")) {
                        let address = base
                            .join(image.value().attr("src").unwrap())
                            .unwrap()
                            .to_string();
                        let alt = image.value().attr("alt").map(|x| x.to_owned());
                        images.push(ImageRef::new(post_id, date, address, alt));
                    }
                }

                if seen.len() - start_length != 10 {
                    break;
                }
            }

            images
        })
        .collect::<Vec<_>>();

    println!("[*] Found {} images", images.len());
    let downloaded = images
        .par_iter()
        .progress_count(images.len() as u64)
        .filter_map(|x| {
            let raw = match ureq::get(&x.address).call() {
                Ok(x) => x,
                Err(..) => return None,
            };

            let content = raw.header("Content-Type").unwrap();
            if raw.status() != 200 || !BMP_TYPES.contains(&content) {
                return None;
            }

            let mut slice = Vec::new();
            raw.into_reader().read_to_end(&mut slice).unwrap();

            let Ok(bmp) = RawBmp::from_slice(&slice) else {
                return None;
            };

            if bmp.header().bpp != Bpp::Bits1 || bmp.header().image_size != Size::new(400, 240) {
                return None;
            }

            fs::write(format!("{OUT_DIR}/{}.bmp", x.id), slice).unwrap();
            Some(x)
        })
        .collect::<Vec<_>>();

    println!("[*] Downloaded {} images", downloaded.len());
    let date = chrono::offset::Local::now().with_timezone(&Utc);
    let info_file = File::create_new(format!("{OUT_DIR}/info.json"))?;
    serde_json::to_writer(
        info_file,
        &json!({
            "date": date,
            "images": downloaded
        }),
    )?;
    Ok(())
}

#[derive(Debug, Serialize, PartialEq, Eq, Hash)]
struct ImageRef {
    id: Uuid,
    post: u32,
    date: DateTime<Utc>,
    address: String,
    alt: Option<String>,
}

impl ImageRef {
    pub fn new(post: u32, date: DateTime<Utc>, address: String, alt: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            post,
            date,
            address,
            alt,
        }
    }
}

// All files: https://forum.swissmicros.com/download/file.php?id=<NUM>
