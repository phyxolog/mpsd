use aho_corasick::AhoCorasick;
use bytes::Bytes;
use colored::Colorize;
use memmap2::Mmap;
use ranges::Ranges;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::iter::Iterator;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use detector::{
    AacDetector, BitmapDetector, DetectOptions, Detector, Mp3Detector, OggDetector,
    RiffWaveDetector, StreamType,
};

mod cli;
mod detector;
mod extractor;

struct Args {
    is_extract: bool,
    detect_options: DetectOptions,
    file_path: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    patterns: HashMap<Bytes, Vec<StreamType>>,
}

struct State {
    is_extract: bool,
    processed_regions: Ranges<usize>,
    total_stream_count: usize,
    total_stream_size: usize,
}

struct Summary {
    process_time: Duration,
    processed_bytes: usize,
    total_stream_size: usize,
    total_stream_count: usize,
}

fn handle_offset(
    buffer: &Mmap,
    offset: usize,
    stream_types: &Vec<StreamType>,
    extractor: &mpsc::Sender<(usize, usize, StreamType)>,
    detect_options: &DetectOptions,
    state: &mut State,
) {
    for x in stream_types {
        if (*state).processed_regions.contains(&offset) {
            return;
        }

        let detector: Box<dyn Detector> = match x {
            StreamType::Ogg => Box::new(OggDetector),
            StreamType::Bitmap => Box::new(BitmapDetector),
            StreamType::RiffWave => Box::new(RiffWaveDetector),
            StreamType::Aac => Box::new(AacDetector),
            StreamType::Mp3 => Box::new(Mp3Detector),
        };

        if let Some((offset, size)) = detector.detect(buffer, offset, detect_options) {
            (*state).total_stream_count += 1;
            (*state).total_stream_size += size;
            (*state).processed_regions.insert(offset..=(offset + size));

            if state.is_extract {
                extractor
                    .send((offset, size, *x))
                    .expect("could not synchronize threads");
            }

            println!("--> Found {:?} stream @ {} ({} bytes)", x, offset, size)
        }
    }
}

fn run(args: Args) -> Summary {
    let file_path = args.file_path.expect("file path is empty");
    let file = File::open(file_path).expect("failed to open the file");

    let mmap = Arc::new(unsafe { Mmap::map(&file).expect("failed to map the file") });

    let mut output_dir: PathBuf = PathBuf::new();

    if args.is_extract {
        output_dir = args.output_dir.expect("output directory is empty");
    }

    let output_dir_cloned = output_dir.clone();

    if args.is_extract {
        fs::create_dir_all(output_dir).expect("could not create directory for extracting files");
    }

    let start_time = Instant::now();
    let (ssx, drx) = mpsc::channel();

    let (onebyte_patterns, patterns): (Vec<Bytes>, Vec<Bytes>) = args
        .patterns
        .keys()
        .cloned()
        .into_iter()
        .partition(|x| x.len() == 1);

    let ac = AhoCorasick::new(&patterns).expect("could not initiate AhoCorasick search engine");

    let ssx_cloned = ssx.clone();
    let mmap_cloned = Arc::clone(&mmap);
    let patterns_cloned = args.patterns.clone();

    let scanner = thread::spawn(move || {
        if ac.patterns_len() > 0 {
            for c in ac.find_iter(&*mmap_cloned) {
                let pattern = &patterns[c.pattern()];

                if let Some(stream_types) = patterns_cloned.get(pattern) {
                    ssx_cloned
                        .send((c.start(), stream_types.clone()))
                        .expect("could not synchronize threads");
                }
            }
        }
    });

    let onebyte_ac =
        AhoCorasick::new(&onebyte_patterns).expect("could not initiate AhoCorasick search engine");

    let onebyte_ssx_cloned = ssx.clone();
    let onebyte_mmap_cloned = Arc::clone(&mmap);
    let patterns_cloned = args.patterns.clone();

    let onebyte_scanner = thread::spawn(move || {
        if onebyte_ac.patterns_len() > 0 {
            for c in onebyte_ac.find_iter(&*onebyte_mmap_cloned) {
                let pattern = &onebyte_patterns[c.pattern()];

                if let Some(stream_types) = patterns_cloned.get(pattern) {
                    onebyte_ssx_cloned
                        .send((c.start(), stream_types.clone()))
                        .expect("could not synchronize threads");
                }
            }
        }
    });

    drop(ssx);

    let mmap_cloned = Arc::clone(&mmap);

    let state = Arc::new(Mutex::new(State {
        processed_regions: Ranges::new(),
        total_stream_size: 0,
        total_stream_count: 0,
        is_extract: args.is_extract,
    }));

    let state_cloned = Arc::clone(&state);
    let (esx, erx) = mpsc::channel();
    let esx_cloned = esx.clone();

    let detector = thread::spawn(move || {
        let mut state = state_cloned.lock().expect("could not lock the state");

        for (offset, stream_types) in drx {
            handle_offset(
                &mmap_cloned,
                offset,
                &stream_types,
                &esx_cloned,
                &args.detect_options,
                &mut state,
            );
        }
    });

    drop(esx);

    let mmap_cloned = Arc::clone(&mmap);

    let extractor = thread::spawn(move || {
        for (offset, size, stream_type) in erx {
            extractor::extract(&mmap_cloned, offset, size, &stream_type, &output_dir_cloned);
        }
    });

    scanner.join().expect("scanner thread panicked");
    detector.join().expect("detector thread panicked");
    extractor.join().expect("extractor thread panicked");

    onebyte_scanner
        .join()
        .expect("onebyte scanner thread panicked");

    let state = state.lock().expect("could not lock the state");

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
    println!("-> Found streams: {}", summary.total_stream_count);
    println!(
        "-> Size of found streams: {} ({} bytes)",
        humanize_size(summary.total_stream_size),
        summary.total_stream_size
    );
}

fn main() {
    let cli_args: cli::Cli = cli::parse();

    let detect_options = DetectOptions {
        mpeg_min_frames: cli_args.mpeg_min_frames,
        mpeg_max_frames: cli_args.mpeg_max_frames,
    };

    let mut patterns: HashMap<Bytes, Vec<StreamType>> = HashMap::from([
        (Bytes::from("OggS"), vec![StreamType::Ogg]),
        (Bytes::from("BM"), vec![StreamType::Bitmap]),
        (Bytes::from("RIFF"), vec![StreamType::RiffWave]),
        (
            Bytes::from(&b"\xFF"[..]),
            vec![StreamType::Aac, StreamType::Mp3],
        ),
    ]);

    patterns.retain(|_, v| match v.as_slice() {
        [StreamType::Ogg] => cli_args.detect_ogg != 0,
        [StreamType::Bitmap] => cli_args.detect_bmp != 0,
        [StreamType::RiffWave] => cli_args.detect_wav != 0,
        [StreamType::Aac, StreamType::Mp3] => {
            if cli_args.detect_aac != 0 && cli_args.detect_mp3 != 0 {
                return true;
            }

            if cli_args.detect_aac != 0 {
                *v = vec![StreamType::Aac];
                return true;
            }

            if cli_args.detect_mp3 != 0 {
                *v = vec![StreamType::Mp3];
                return true;
            }

            return false;
        }
        _ => false,
    });

    let mut args = Args {
        patterns,
        detect_options,
        output_dir: None,
        file_path: None,
        is_extract: false,
    };

    match cli_args.command {
        cli::Commands::Scan { file_path } => {
            println!("-> Scanning...");
            args.file_path = Some(PathBuf::from(file_path));
            let summary: Summary = run(args);
            print_summary(&summary);
        }
        cli::Commands::Extract {
            file_path,
            output_dir,
        } => {
            println!("-> Scanning and extracting...");
            args.is_extract = true;
            args.file_path = Some(PathBuf::from(file_path));
            args.output_dir = Some(PathBuf::from(output_dir));
            let summary: Summary = run(args);
            print_summary(&summary);
        }
    }
}
