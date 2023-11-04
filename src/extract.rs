use std::{collections::HashSet, sync::Arc};

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use indicatif::ParallelProgressIterator;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use scraper::{ElementRef, Html};
use url::Url;

use crate::{
    util::{selector, RawUrl},
    AssetRef, Post,
};

type Extractor = fn(&mut Vec<AssetRef>, &Url, ElementRef<'_>, Arc<Post>);
const EXTRACTORS: &[Extractor] = &[extract_images, extract_zips];

fn extract_images(
    assets: &mut Vec<AssetRef>,
    base_url: &Url,
    post: ElementRef<'_>,
    post_info: Arc<Post>,
) {
    for image in post.select(selector!("img.postimage")) {
        let image_src = image.value().attr("src").unwrap();
        let address = base_url.join(image_src).unwrap();
        let alt = image.value().attr("alt").map(|x| x.to_owned());
        assets.push(AssetRef {
            post: post_info.clone(),
            address,
            alt,
        });
    }
}

fn extract_zips(
    assets: &mut Vec<AssetRef>,
    base_url: &Url,
    post: ElementRef<'_>,
    post_info: Arc<Post>,
) {
    for link in post.select(selector!("a.postlink")) {
        let href = link.value().attr("href").unwrap();
        let address = base_url.join(href).unwrap();
        assets.push(AssetRef {
            post: post_info.clone(),
            address,
            alt: None,
        });
    }
}

pub fn get_topics(base_url: &Url) -> Result<HashSet<RawUrl>> {
    let mut topics = HashSet::new();
    for page in 0.. {
        let url = format!("{base_url}/viewforum.php?f=14&start={}", 25 * page);
        let res = ureq::get(&url).call()?;
        if res.status() != 200 {
            bail!("Failed to fetch page #{page}: {}", res.status());
        }

        let dom = Html::parse_document(&res.into_string()?);
        let posts = dom
            .select(selector!(".topictitle"))
            .map(|x| x.value().attr("href").unwrap()[2..].into())
            .collect::<Vec<_>>();
        let count = posts.len();
        topics.extend(posts);

        if count != 25 {
            break;
        }
    }

    Ok(topics)
}

pub fn get_assets(base_url: &Url, topics: HashSet<RawUrl>) -> Vec<AssetRef> {
    topics
        .par_iter()
        .progress_count(topics.len() as u64)
        .flat_map(|post| {
            let mut assets = Vec::new();
            let mut seen = HashSet::new();

            'outer: for page in 0.. {
                let url = format!("{base_url}/{post}&start={}", page * 10);
                let post_id = post
                    .split("t=")
                    .nth(1)
                    .unwrap()
                    .split('&')
                    .next()
                    .unwrap()
                    .parse::<u32>()
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

                    let post_info = Arc::new(Post {
                        post: post_id,
                        date,
                    });

                    for extract in EXTRACTORS {
                        extract(&mut assets, base_url, post, post_info.clone());
                    }
                }

                if seen.len() - start_length != 10 {
                    break;
                }
            }

            assets
        })
        .collect::<Vec<_>>()
}
