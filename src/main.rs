use std::{collections::HashSet, fs, path::Path};

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use embedded_graphics::prelude::Size;
use indicatif::ParallelProgressIterator;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use scraper::Html;
use tinybmp::{Bpp, RawBmp};
use url::Url;

const BASE_URL: &str = "https://forum.swissmicros.com";
const OUT_DIR: &str = "out";

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

    let mut links = HashSet::new();
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
        links.extend(posts);

        if count != 25 {
            break;
        }
    }

    println!("Found {} posts", links.len());
    let images = links
        .par_iter()
        .progress_count(links.len() as u64)
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
                    if seen.contains(id) {
                        break 'outer;
                    } else {
                        seen.insert(id.to_owned());
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
                        images.push(ImageRef {
                            post: post_id,
                            date,
                            address,
                            alt,
                        });
                    }
                }

                if seen.len() - start_length != 10 {
                    break;
                }
            }

            images
        })
        .collect::<Vec<_>>();

    println!("Found {} images", images.len());
    images
        .par_iter()
        .progress_count(images.len() as u64)
        .for_each(|x| {
            let raw = match ureq::get(&x.address).call() {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("Failed to fetch image {}: {}", x.address, e);
                    return;
                }
            };
            if raw.status() != 200 {
                panic!("Failed to fetch image {}: {}", x.address, raw.status());
            }

            if raw.header("Content-Type").unwrap() != "image/bmp" {
                return;
            }

            let mut slice = Vec::new();
            raw.into_reader().read_to_end(&mut slice).unwrap();

            let bmp = RawBmp::from_slice(&slice).unwrap();
            if bmp.header().bpp != Bpp::Bits1 || bmp.header().image_size != Size::new(400, 240) {
                return;
            }

            fs::write(
                format!(
                    "{}/{}-{}{}.bmp",
                    OUT_DIR,
                    x.date.format("%Y-%m-%d_%H-%M-%S"),
                    x.post,
                    if let Some(i) = &x.alt {
                        String::from("-") + &urlencoding::encode(i)
                    } else {
                        String::new()
                    }
                ),
                slice,
            )
            .unwrap();
        });

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct ImageRef {
    post: u32,
    date: DateTime<Utc>,
    address: String,
    alt: Option<String>,
}

// All files: https://forum.swissmicros.com/download/file.php?id=<NUM>
