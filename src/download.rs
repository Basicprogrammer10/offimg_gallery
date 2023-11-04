use std::{
    fs,
    io::{Cursor, Read},
    ops::Deref,
    path::Path,
};

use embedded_graphics::prelude::Size;
use indicatif::ParallelProgressIterator;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use tinybmp::{Bpp, RawBmp};
use uuid::Uuid;
use zip::ZipArchive;

use crate::{AssetRef, ImageRef};

type Downloader = fn(&Path, &AssetRef, &[u8]) -> Option<Vec<ImageRef>>;
const DOWNLOADERS: &[(&[&str], Downloader)] =
    &[(BMP_TYPES, download_bitmap), (ZIP_TYPES, download_zip)];

const BMP_TYPES: &[&str] = &["image/bmp", "image/x-ms-bmp"];
fn download_bitmap(out_dir: &Path, asset: &AssetRef, slice: &[u8]) -> Option<Vec<ImageRef>> {
    Some(vec![save_bitmap(out_dir, asset, slice)?])
}

const ZIP_TYPES: &[&str] = &["application/zip", "application/octet-stream"];
fn download_zip(out_dir: &Path, asset: &AssetRef, slice: &[u8]) -> Option<Vec<ImageRef>> {
    let cursor = Cursor::new(slice);
    let Ok(mut archive) = ZipArchive::new(cursor) else {
        return None;
    };

    let mut images = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name().ends_with(".bmp") {
            let mut slice = Vec::new();
            file.read_to_end(&mut slice).unwrap();

            if let Some(img) = save_bitmap(out_dir, asset, &slice) {
                images.push(img);
            }
        }
    }

    Some(images)
}

pub fn download(out_dir: &Path, assets: Vec<AssetRef>) -> Vec<ImageRef> {
    assets
        .par_iter()
        .progress_count(assets.len() as u64)
        .filter_map(|asset| {
            let Ok(raw) = ureq::get(asset.address.to_string().as_str()).call() else {
                return None;
            };

            let content = raw.header("Content-Type").unwrap().to_owned();
            if raw.status() != 200 {
                return None;
            }

            let mut slice = Vec::new();
            raw.into_reader().read_to_end(&mut slice).ok()?;

            for downloader in DOWNLOADERS {
                if !downloader.0.contains(&content.as_str()) {
                    continue;
                }

                if let Some(img) = downloader.1(out_dir, asset, &slice) {
                    return Some(img);
                }
            }

            None
        })
        .flatten()
        .collect::<Vec<_>>()
}

fn save_bitmap(out_dir: &Path, asset: &AssetRef, slice: &[u8]) -> Option<ImageRef> {
    let Ok(bmp) = RawBmp::from_slice(slice) else {
        return None;
    };

    if bmp.header().bpp != Bpp::Bits1 || bmp.header().image_size != Size::new(400, 240) {
        return None;
    }

    let id = Uuid::new_v4();
    let path = out_dir.join(format!("{id}.bmp"));
    fs::write(path, slice).unwrap();
    Some(ImageRef {
        uuid: id,
        post: asset.post.deref().clone(),
        alt: asset.alt.clone(),
    })
}
