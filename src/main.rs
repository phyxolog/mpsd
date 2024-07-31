use aho_corasick::AhoCorasick;
use bytes::Bytes;
use colored::Colorize;
use detector::{Detector, StreamType};
use lazy_static::lazy_static;
use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::File;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

mod cli;
mod detector;

static TOTAL_STREAM_COUNT: AtomicUsize = AtomicUsize::new(0);
static TOTAL_STREAM_SIZE: AtomicUsize = AtomicUsize::new(0);

struct Summary {
    process_time: Duration,
    processed_bytes: usize,
    total_stream_count: usize,
    total_stream_size: usize,
}

lazy_static! {
    static ref PATTERNS: HashMap<Bytes, StreamType> = {
        HashMap::from([
            (Bytes::from("OggS"), StreamType::Ogg),
            (Bytes::from("BM"), StreamType::Bitmap),
            (Bytes::from("RIFF"), StreamType::RiffWave),
        ])
    };
}

fn handle_offset(buffer: &Mmap, offset: usize, media_type: &StreamType) {
    let detector: Box<dyn Detector> = match media_type {
        StreamType::Ogg => Box::new(detector::OggDetector),
        StreamType::Bitmap => Box::new(detector::BitmapDetector),
        StreamType::RiffWave => Box::new(detector::RiffWaveDetector),
    };

    match detector.detect(buffer, offset) {
        Some((offset, size)) => {
            TOTAL_STREAM_COUNT.fetch_add(1, Ordering::Relaxed);
            TOTAL_STREAM_SIZE.fetch_add(size, Ordering::Relaxed);

            println!(
                "--> Found {:?} stream @ {:#016X} ({} bytes)",
                media_type, offset, size
            )
        }
        _ => {}
    }
}

fn run(path: &str) -> Summary {
    let file = File::open(path).expect("failed to open the file");

    let mmap = Arc::new(unsafe { Mmap::map(&file).expect("failed to map the file") });

    let start_time = Instant::now();

    let (sx, rx) = mpsc::channel();
    let patterns: Vec<&Bytes> = PATTERNS.keys().collect();

    let ac = AhoCorasick::new(&patterns).unwrap();

    let sx_cloned = sx.clone();
    let mmap_cloned = Arc::clone(&mmap);

    let sender = thread::spawn(move || {
        for c in ac.find_iter(&*mmap_cloned) {
            let pattern = patterns[c.pattern()];

            sx_cloned
                .send((c.start(), PATTERNS.get(pattern).unwrap()))
                .unwrap();
        }
    });

    drop(sx);

    let mmap_cloned = Arc::clone(&mmap);

    let receiver = thread::spawn(move || {
        for (offset, media_type) in rx {
            handle_offset(&mmap_cloned, offset, media_type);
        }
    });

    sender.join().unwrap();
    receiver.join().unwrap();

    Summary {
        process_time: start_time.elapsed(),
        processed_bytes: mmap.len(),
        total_stream_size: TOTAL_STREAM_SIZE.load(Ordering::Relaxed),
        total_stream_count: TOTAL_STREAM_COUNT.load(Ordering::Relaxed),
    }
}

fn humanize_size(bytes: usize) -> String {
    const UNITS: [&str; 7] = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];

    let exp = (bytes as f64).log(1024.0).floor() as usize;
    let size_in_units = bytes as f64 / 1024_f64.powi(exp as i32);

    format!("{:.2} {}", size_in_units, UNITS[exp])
}

fn print_summary(summary: &Summary) {
    let elapsed_seconds = summary.process_time.as_secs_f64();
    let processed_bytes = summary.processed_bytes as f64;
    let speed_mbps = (processed_bytes / (1024.0 * 1024.0)) / elapsed_seconds;

    println!("\n{}", "Summary:\n".bold().underline());
    println!("-> Process time: {:?}", summary.process_time);
    println!("-> Speed: {:.2} MB/s", speed_mbps);
    println!("-> Found media streams: {}", summary.total_stream_count);
    println!(
        "-> Size of found media streams: {} ({} bytes)",
        humanize_size(summary.total_stream_size),
        summary.total_stream_size
    );
}

fn main() {
    let args: cli::Cli = cli::parse();

    match args.command {
        cli::Commands::Scan { file_path } => {
            println!("-> Scanning...");

            let summary: Summary = run(&file_path);
            print_summary(&summary);
        }
        cli::Commands::Extract { file_path, out_dir } => {
            println!("Pushing {file_path} {out_dir}");
        }
    }
}
