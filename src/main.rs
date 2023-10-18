use std::{collections::HashSet, fs, path::Path};

use anyhow::{bail, Result};
use indicatif::ParallelProgressIterator;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use scraper::Html;
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
            let mut images = HashSet::new();
            for page in 0.. {
                let url = format!("{BASE_URL}/{post}&start={}", page * 10);
                let res = ureq::get(&url).call().unwrap();
                if res.status() != 200 {
                    panic!("Failed to fetch post {}: {}", post, res.status());
                }

                let dom = Html::parse_document(&res.into_string().unwrap());
                let page_count = dom.select(selector!(".post")).count();
                let imgs = dom
                    .select(selector!("img.postimage"))
                    .map(|x| ImageRef {
                        address: base
                            .join(x.value().attr("src").unwrap())
                            .unwrap()
                            .to_string(),
                        alt: x.value().attr("alt").unwrap().to_owned(),
                    })
                    .collect::<Vec<_>>();
                images.extend(imgs);

                if page_count != 10 {
                    break;
                }
            }

            images
        })
        .collect::<HashSet<_>>();

    println!("Found {} images", images.len());

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct ImageRef {
    address: String,
    alt: String,
}

// All files: https://forum.swissmicros.com/download/file.php?id=<NUM>
