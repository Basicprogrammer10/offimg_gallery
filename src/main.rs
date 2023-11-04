#![feature(decl_macro)]

use std::{
    fs::{self, File},
    io::Write,
    path::Path,
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

    if !args.out_dir.exists() {
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
    let info_file = File::create(args.out_dir.join("info.json"))?;
    serde_json::to_writer(
        info_file,
        &json!({
            "date": date,
            "images": images
        }),
    )?;

    if !args.keep {
        for file in fs::read_dir(&args.out_dir)? {
            let file = file?;
            let name = file.file_name();
            let name = name.to_string_lossy();
            if file.file_type()?.is_file()
                && name.ends_with(".bmp")
                && images.iter().all(|i| name != format!("{}.bmp", i.uuid))
            {
                fs::remove_file(file.path())?;
            }
        }
    }

    Ok(())
}

fn compress(out_dir: &Path, images: &[ImageRef]) -> Result<()> {
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

#[derive(Debug, Serialize, Clone)]
pub struct Post {
    post: u32,
    date: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AssetRef {
    post: Arc<Post>,
    address: Url,
    alt: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ImageRef {
    post: Post,
    uuid: Uuid,
    alt: Option<String>,
}
