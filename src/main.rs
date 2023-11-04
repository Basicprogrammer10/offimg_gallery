#![feature(file_create_new)]
#![feature(decl_macro)]

use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    sync::Arc,
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::Serialize;
use serde_json::json;
use url::Url;
use uuid::Uuid;
use zip::ZipWriter;

use crate::args::Args;

mod args;
mod download;
mod extract;
mod util;

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.keep {
        if args.out_dir.exists() {
            fs::remove_dir_all(&args.out_dir)?;
        }
        fs::create_dir(&args.out_dir)?;
    }

    let topics = extract::get_topics(&args.base_url)?;
    println!("[*] Found {} topics", topics.len());
    let assets = extract::get_assets(&args.base_url, topics);
    println!("[*] Found {} assets", assets.len());
    let images = download::download(&args.out_dir, assets);
    println!("[*] Downloaded {} images", images.len());

    if !args.no_compress {
        match compress(&args.out_dir, &images) {
            Ok(()) => println!("[*] Compressed images"),
            Err(e) => println!("[!] Failed to compress images: {}", e),
        };
    }

    let date = chrono::offset::Local::now().with_timezone(&Utc);
    let info_file = File::create_new(args.out_dir.join("info.json"))?;
    serde_json::to_writer(
        info_file,
        &json!({
            "date": date,
            "images": images
        }),
    )?;
    Ok(())
}

fn compress(out_dir: &PathBuf, images: &[ImageRef]) -> Result<()> {
    let file = File::create(out_dir.join("all.zip"))?;
    let mut zip = ZipWriter::new(file);

    for i in images {
        zip.start_file(format!("{}.bmp", i.uuid), Default::default())?;
        let data = fs::read(out_dir.join(format!("{}.bmp", i.uuid)))?;
        zip.write_all(&data)?;
    }

    zip.finish()?;
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
