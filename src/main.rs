mod decoder;
mod encoder;
mod error;

use clap::{Parser, Subcommand};
use colored::*;
use error::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rusty-storage")]
#[command(about = "Convert files to videos and back", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode a file to video
    Encode {
        /// Input file to encode
        #[arg(short, long)]
        input: PathBuf,

        /// Output video file
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Decode a video back to file
    Decode {
        /// Input video file
        #[arg(short, long)]
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encode { input, output } => {
            println!("{}", "Starting encoding process...".green());

            // Create progress bar
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap(),
            );
            pb.set_message("Encoding file to video...");

            // Encode file to video
            let encoder = encoder::Encoder::new(output.to_str().unwrap());
            encoder.encode_file(&input)?;

            pb.finish_with_message(format!(
                "{}",
                format!(
                    "Encoding complete! Video saved to: {}",
                    output.display()
                )
                .green()
            ));
        }
        Commands::Decode { input, output } => {
            println!("{}", "Starting decoding process...".green());

            // Create progress bar
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap(),
            );
            pb.set_message("Decoding video to file...");

            // Decode video to file
            let decoder = decoder::Decoder::new(input.to_str().unwrap());
            decoder.decode_file(&output)?;

            pb.finish_with_message(format!(
                "{}",
                format!(
                    "Decoding complete! File saved to: {}",
                    output.display()
                )
                .green()
            ));
        }
    }

    Ok(())
}