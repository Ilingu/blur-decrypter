use clap::{Parser, Subcommand};
use core::panic;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use blur::{Image, ImageBlur};
use text2img::text_to_image;

mod blur;
mod text2img;

/// Program to crack/decrypt blurred text (previously blurred by the same program)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Blur a piece of text
    Blur {
        /// Text to blur
        #[arg(short, long)]
        text: String,

        /// Performs a Gaussian blur on this image. sigma is a measure of how much to blur by.
        #[arg(short, long, default_value_t = 2.0)]
        sigma: f32,

        /// Output **PNG** file (should not already exist)
        #[arg(short, long)]
        output: String,
    },
    /// Unblur the previoulsy blured text (see `blur` command)
    Unblur {
        /// Input to the image text to unblur
        #[arg(short, long)]
        input: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Blur {
            text,
            sigma,
            output,
        } => {
            let dirs = output.split('/').collect::<Vec<_>>();
            fs::create_dir_all(dirs[..dirs.len() - 1].join("/"))
                .expect("Failed to create output directory");

            let ext = Path::new(&output)
                .extension()
                .expect("Failed to read output file extension");
            if ext != "png" {
                panic!("Wrong output file extension. Expected PNG");
            }

            let img_bytes = text_to_image(&text).unwrap();
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&output)
                .expect("Failed to open output file");
            file.write_all(&img_bytes)
                .expect("Failed to write to output file");

            let image: Image = output.to_owned().try_into().unwrap();
            let blurred_image = image.0.blur(sigma);
            blurred_image
                .save_with_format(output, image::ImageFormat::Png)
                .expect("Failed to save");
        }
        Commands::Unblur { input } => {
            let image: Image = input.try_into().unwrap();
            let unblurred_text = image.unblur_text().expect("Failed to unblur");
            println!("Unblurred text: {unblurred_text}");
        }
    }
}
