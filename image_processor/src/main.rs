use std::fs;
use clap::{ Parser };

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
    println!("Hello, world!");

    let cli = Cli::try_parse()?;
    let image_bytes = fs::read(&cli.input)?;
    let image = image::load_from_memory(&image_bytes)?;

    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels: Vec<u8> = rgba.into_raw();

    //TODO вызов функции обработки через плагины

    Ok(())
}
