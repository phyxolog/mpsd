use clap::{Parser, Subcommand};
use std::ops::RangeInclusive;

/// Multimedia streams detector
#[derive(Debug, Parser)]
#[command(long_about = None)]
pub struct Cli {
    /// Enable RIFF WAVE detection
    #[arg(long = "wav", global = true, value_parser = validate_bool, default_value_t = 1)]
    pub detect_wav: u8,

    /// Enable Ogg detection
    #[arg(long = "ogg", global = true, value_parser = validate_bool, default_value_t = 1)]
    pub detect_ogg: u8,

    /// Enable Bitmap (BMP) detection
    #[arg(long = "bmp", global = true, value_parser = validate_bool, default_value_t = 1)]
    pub detect_bmp: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Scan input file
    #[command(arg_required_else_help = true)]
    Scan {
        /// Path for the input file
        file_path: String,
    },
    /// Extract media streams from input file
    #[command(arg_required_else_help = true)]
    Extract {
        /// Path for the input file
        file_path: String,

        /// Path for the output folder (for extracted files)
        out_dir: String,
    },
}

pub fn parse() -> Cli {
    let args = Cli::parse();
    return args;
}

fn validate_bool(s: &str) -> Result<u8, String> {
    let value: usize = s.parse().map_err(|_| format!("`{s}` isn't a number"))?;
    let range: RangeInclusive<usize> = 0..=1;

    if range.contains(&value) {
        Ok(value as u8)
    } else {
        Err(format!("should be 0 or 1"))
    }
}
