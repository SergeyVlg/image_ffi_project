mod plugin_loader;
mod error;

use crate::plugin_loader::Plugin;
use clap::Parser;
use std::{fs, process};
use std::path::PathBuf;
use libloading::library_filename;
use crate::error::ProcessError;

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
    plugin_path: PathBuf,
}

fn main() {

    if let Err(e) = run() {
        match e {
            ProcessError::ParseError(clap_err) => {
                clap_err.exit(); //для красивого вывода в консоль через Display
            }
            other_error => {
                eprintln!("{}", other_error);
                process::exit(1);
            }
        }
    }
}

fn run() -> Result<(), ProcessError> {
    let cli = Cli::try_parse()?;
    let image_bytes = fs::read(&cli.input)?;
    let image = image::load_from_memory(&image_bytes)?;

    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut pixels = rgba.into_raw();

    let plugin_file_name = format!("{}_plugin", cli.plugin);
    let cross_plugin_file_name = library_filename(plugin_file_name);

    let mut plugin_path = cli.plugin_path;
    plugin_path.push(cross_plugin_file_name);

    let plugin = Plugin::new(plugin_path)?;
    let params = fs::read_to_string(&cli.params)?;

    plugin.process_image(width, height, &mut pixels, &params)?;
    println!("Processing done, saving...");

    let processed_image = image::RgbaImage::from_raw(width, height, pixels)
        .ok_or_else(|| ProcessError::CorruptedImage)?;

    processed_image.save(&cli.output)?;
    println!("Image successfully saved.");

    Ok(())
}