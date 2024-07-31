use aho_corasick::AhoCorasick;
use bytes::Bytes;
use colored::Colorize;
use detector::{Detector, StreamType};
use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::File;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

mod cli;
mod detector;

struct State {
    total_stream_count: usize,
    total_stream_size: usize,
    skip_offset: usize,
}

struct Summary {
    process_time: Duration,
    processed_bytes: usize,
    total_stream_count: usize,
    total_stream_size: usize,
}

struct Args<'a> {
    file_path: Option<&'a String>,
    output_dir: Option<&'a String>,
    patterns: &'a HashMap<Bytes, StreamType>,
}

fn handle_offset(buffer: &Mmap, offset: usize, stream_type: &StreamType, state: &mut State) {
    let detector: Box<dyn Detector> = match stream_type {
        StreamType::Ogg => Box::new(detector::OggDetector),
        StreamType::Bitmap => Box::new(detector::BitmapDetector),
        StreamType::RiffWave => Box::new(detector::RiffWaveDetector),
        StreamType::Aac => Box::new(detector::AacDetector),
    };

    if offset <= state.skip_offset {
        return;
    }

    match detector.detect(buffer, offset) {
        Some((offset, size)) => {
            (*state).total_stream_size += size;
            (*state).total_stream_count += 1;
            (*state).skip_offset = offset + size;

            println!(
                "--> Found {:?} stream @ {:#016X} ({} bytes)",
                stream_type, offset, size
            )
        }
        _ => {}
    }
}

fn run(args: &Args) -> Summary {
    let file_path = args.file_path.expect("file path is empty");
    let file = File::open(file_path).expect("failed to open the file");

    let mmap = Arc::new(unsafe { Mmap::map(&file).expect("failed to map the file") });

    let start_time = Instant::now();

    let (ssx, drx) = mpsc::channel();
    let patterns: Vec<Bytes> = args.patterns.keys().cloned().collect();

    let ac = AhoCorasick::new(&patterns).unwrap();

    let ssx_cloned = ssx.clone();
    let mmap_cloned = Arc::clone(&mmap);
    let patterns_cloned = args.patterns.clone();

    let scanner = thread::spawn(move || {
        for c in ac.find_iter(&*mmap_cloned) {
            let pattern = &patterns[c.pattern()];

            if let Some(stream_type) = patterns_cloned.get(pattern) {
                ssx_cloned.send((c.start(), stream_type.clone())).unwrap();
            }
        }
    });

    drop(ssx);

    let mmap_cloned = Arc::clone(&mmap);

    let state = Arc::new(Mutex::new(State {
        total_stream_size: 0,
        total_stream_count: 0,
        skip_offset: 0,
    }));

    let state_cloned = Arc::clone(&state);

    let detector = thread::spawn(move || {
        let mut state = state_cloned.lock().unwrap();

        for (offset, stream_type) in drx {
            handle_offset(&mmap_cloned, offset, &stream_type, &mut state);
        }
    });

    scanner.join().unwrap();
    detector.join().unwrap();

    let state = state.lock().unwrap();

    Summary {
        process_time: start_time.elapsed(),
        processed_bytes: mmap.len(),
        total_stream_size: (*state).total_stream_size,
        total_stream_count: (*state).total_stream_count,
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
    let cli_args: cli::Cli = cli::parse();

    let mut patterns: HashMap<Bytes, StreamType> = HashMap::from([
        (Bytes::from("OggS"), StreamType::Ogg),
        (Bytes::from("BM"), StreamType::Bitmap),
        (Bytes::from("RIFF"), StreamType::RiffWave),
        (Bytes::from(&b"\xFF"[..]), StreamType::Aac),
    ]);

    patterns.retain(|_, v| match v {
        StreamType::Bitmap => cli_args.detect_bmp != 0,
        StreamType::Ogg => cli_args.detect_ogg != 0,
        StreamType::RiffWave => cli_args.detect_wav != 0,
        StreamType::Aac => true,
    });

    let mut args = Args {
        patterns: &patterns,
        output_dir: None,
        file_path: None,
    };

    match cli_args.command {
        cli::Commands::Scan { file_path } => {
            println!("-> Scanning...");
            args.file_path = Some(&file_path);
            let summary: Summary = run(&args);
            print_summary(&summary);
        }
        cli::Commands::Extract {
            file_path,
            output_dir,
        } => {
            println!("-> Scanning and extracting...");
            args.file_path = Some(&file_path);
            args.output_dir = Some(&output_dir);
            let summary: Summary = run(&args);
            print_summary(&summary);
        }
    }
}
