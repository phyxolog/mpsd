use clap::value_parser;
use clap::{Parser, Subcommand};

/// Multi-Pattern Streams Detector
#[derive(Debug, Parser)]
#[command(long_about = None)]
pub struct Cli {
    /// Enable WAV (RIFF WAVE PCM) detection
    #[arg(long = "wav", global = true, value_parser = value_parser!(u8).range(0..=1), default_value_t = 1)]
    pub detect_wav: u8,

    /// Enable OGG detection
    #[arg(long = "ogg", global = true, value_parser = value_parser!(u8).range(0..=1), default_value_t = 1)]
    pub detect_ogg: u8,

    /// Enable BMP (Windows BitMaP) detection
    #[arg(long = "bmp", global = true, value_parser = value_parser!(u8).range(0..=1), default_value_t = 1)]
    pub detect_bmp: u8,

    /// Enable AAC (ADTS) detection
    #[arg(long = "aac", global = true, value_parser = value_parser!(u8).range(0..=1), default_value_t = 1)]
    pub detect_aac: u8,

    /// Enable MP3 (MPEG-1/2 Audio) detection
    #[arg(long = "mp3", global = true, value_parser = value_parser!(u8).range(0..=1), default_value_t = 1)]
    pub detect_mp3: u8,

    /// Minimum MPEG frames (0 = disabled)
    #[arg(long = "mpeg-min-frames", global = true, default_value_t = 20)]
    pub mpeg_min_frames: u8,

    /// Maximum MPEG frames (0 = disabled)
    #[arg(long = "mpeg-max-frames", global = true, default_value_t = 10000)]
    pub mpeg_max_frames: u16,

    /// Do not print log on each found stream
    #[arg(short = 's', long = "silent", global = true, value_parser = value_parser!(bool), default_value_t = false)]
    pub silent: bool,

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
    /// Extract streams from input file
    #[command(arg_required_else_help = true)]
    Extract {
        /// Path for the input file
        file_path: String,

        /// Path for the output folder (for extracted files)
        output_dir: String,
    },
}

pub fn parse() -> Cli {
    let args = Cli::parse();
    return args;
}
