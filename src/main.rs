#![feature(file_create_new)]
#![feature(decl_macro)]

use std::{
    fs::{self, File},
    path::Path,
    sync::Arc,
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::json;
use url::Url;
use uuid::Uuid;

mod download;
mod extract;
mod util;

const BASE_URL: Lazy<Url> = Lazy::new(|| Url::parse("https://forum.swissmicros.com").unwrap());
const OUT_DIR: &str = "out";

fn main() -> Result<()> {
    if !Path::new(OUT_DIR).exists() {
        fs::create_dir(OUT_DIR)?;
    }

    let topics = extract::get_topics(&BASE_URL)?;
    println!("[*] Found {} topics", topics.len());
    let assets = extract::get_assets(&BASE_URL, topics);
    println!("[*] Found {} assets", assets.len());
    let images = download::download(OUT_DIR, assets);
    println!("[*] Downloaded {} images", images.len());

    let date = chrono::offset::Local::now().with_timezone(&Utc);
    let info_file = File::create_new(format!("{OUT_DIR}/info.json"))?;
    serde_json::to_writer(
        info_file,
        &json!({
            "date": date,
            "images": images
        }),
    )?;
    Ok(())
}

#[derive(Serialize, Clone)]
pub struct Post {
    post: u32,
    date: DateTime<Utc>,
}

pub struct AssetRef {
    post: Arc<Post>,
    address: Url,
    alt: Option<String>,
}

#[derive(Serialize)]
pub struct ImageRef {
    post: Post,
    uuid: Uuid,
    alt: Option<String>,
}
