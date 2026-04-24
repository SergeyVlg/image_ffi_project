mod plugin_loader;
mod error;

use crate::plugin_loader::Plugin;
use clap::Parser;
use std::fs;

#[derive(Parser)]
#[command(name = "image-processor", about = "CLI-утилита для обработки изображений с помощью плагинов", version)]
struct Cli {
    #[arg(long)]
    input: String,
    #[arg(long)]
    output: String,
    #[arg(long)]
    plugin: String,
    #[arg(long)]
    params: String,
    #[arg(long)]
    plugin_path: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::try_parse()?;
    let image_bytes = fs::read(&cli.input)?;
    let image = image::load_from_memory(&image_bytes)?;

    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut pixels = rgba.into_raw();

    let plugin = Plugin::new(&cli.plugin_path)?;
    plugin.process_image(width, height, &mut pixels, &cli.params)?;

    Ok(())
}