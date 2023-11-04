use std::path::PathBuf;

use clap::Parser;
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// The output directory
    pub out_dir: PathBuf,

    /// The base URL of the forum
    #[clap(long, default_value = "https://forum.swissmicros.com", value_parser = Url::parse)]
    pub base_url: Url,

    /// Weather to compress the images into a zip file after downloading
    #[clap(long)]
    pub no_compress: bool,

    /// Weather to keep the output directory if it already exists
    #[clap(long)]
    pub keep: bool,
}
