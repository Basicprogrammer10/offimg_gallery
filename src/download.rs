use std::{fs, io::Read, ops::Deref};

use embedded_graphics::prelude::Size;
use indicatif::ParallelProgressIterator;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use tinybmp::{Bpp, RawBmp};
use uuid::Uuid;

use crate::{AssetRef, ImageRef};

const DOWNLOADERS: &[(
    &[&str],
    fn(&str, &AssetRef, &mut Box<dyn Read + Send + Sync>) -> Option<ImageRef>,
)] = &[(BMP_TYPES, download_bitmap)];

const BMP_TYPES: &[&str] = &["image/bmp", "image/x-ms-bmp"];
fn download_bitmap(out_dir: &str, asset: &AssetRef, reader: &mut impl Read) -> Option<ImageRef> {
    let mut slice = Vec::new();
    reader.read_to_end(&mut slice).unwrap();

    save_bitmap(out_dir, asset, &slice)
}

const ZIP_TYPES: &[&str] = &["application/zip", "application/octet-stream"];
fn download_zip(out_dir: &str, asset: &AssetRef, reader: &mut impl Read) -> Option<ImageRef> {
    todo!()
}

pub fn download(out_dir: &str, assets: Vec<AssetRef>) -> Vec<ImageRef> {
    assets
        .par_iter()
        .progress_count(assets.len() as u64)
        .filter_map(|asset| {
            let Ok(raw) = ureq::get(asset.address.to_string().as_str()).call() else {
                return None;
            };

            let content = raw.header("Content-Type").unwrap();
            if raw.status() != 200 {
                return None;
            }

            for downloader in DOWNLOADERS {
                if downloader.0.contains(&content) {
                    return downloader.1(out_dir, asset, &mut raw.into_reader());
                }
            }

            None
        })
        .collect::<Vec<_>>()
}

fn save_bitmap(out_dir: &str, asset: &AssetRef, slice: &[u8]) -> Option<ImageRef> {
    let Ok(bmp) = RawBmp::from_slice(&slice) else {
        return None;
    };

    if bmp.header().bpp != Bpp::Bits1 || bmp.header().image_size != Size::new(400, 240) {
        return None;
    }

    let id = Uuid::new_v4();
    fs::write(format!("{out_dir}/{}.bmp", id), slice).unwrap();
    Some(ImageRef {
        uuid: id,
        post: asset.post.deref().clone(),
        alt: asset.alt.clone(),
    })
}
