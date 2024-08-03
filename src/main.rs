use aho_corasick::AhoCorasick;
use bytes::Bytes;
use colored::Colorize;
use glob::glob;
use injector::is_mmap_support;
use memmap2::{Mmap, MmapMut};
use range_set_blaze::RangeSetBlaze;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::iter::Iterator;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use detector::{
    AacDetector, BitmapDetector, DetectOptions, Detector, Mp3Detector, OggDetector,
    RiffWaveDetector, StreamMatch, StreamType,
};

mod cli;
mod detector;
mod eraser;
mod extractor;
mod injector;

struct Args {
    silent: bool,
    is_extract: bool,
    erase_sectors: bool,
    file_path: Option<PathBuf>,
    input_dir: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    detect_options: DetectOptions,
    patterns: HashMap<Bytes, Vec<StreamType>>,
}

struct State {
    silent: bool,
    is_extract: bool,
    total_streams_size: usize,
    total_streams_count: usize,
    processed_sectors: RangeSetBlaze<usize>,
}

struct Summary {
    process_time: Duration,
    processed_bytes: usize,
    total_streams_size: usize,
    total_streams_count: usize,
}

fn handle_offset(
    buffer: &Mmap,
    offset: usize,
    stream_types: &Vec<StreamType>,
    extractor: &mpsc::Sender<(usize, usize, String)>,
    detect_options: &DetectOptions,
    state: &mut State,
) {
    for x in stream_types {
        if state.processed_sectors.contains(offset) {
            return;
        }

        let detector: Box<dyn Detector> = match x {
            StreamType::Aac => Box::new(AacDetector),
            StreamType::Ogg => Box::new(OggDetector),
            StreamType::Mp3 => Box::new(Mp3Detector),
            StreamType::Bitmap => Box::new(BitmapDetector),
            StreamType::RiffWave => Box::new(RiffWaveDetector),
        };

        if let Some(StreamMatch { offset, size, ext }) =
            detector.detect(buffer, offset, detect_options)
        {
            let sector = offset..=(offset + size);
            (*state).total_streams_count += 1;
            (*state).total_streams_size += size;
            (*state).processed_sectors.ranges_insert(sector);

            if state.is_extract {
                extractor
                    .send((offset, size, ext.to_string()))
                    .expect("could not synchronize threads");
            }

            if !state.silent {
                println!("--> Found {:?} stream @ {} ({} bytes)", x, offset, size);
            }
        }
    }
}

fn run(args: Args) -> Summary {
    let file_path = args.file_path.expect("file path is not set");

    let file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(file_path)
        .expect("failed to open the file");

    let mmap = Arc::new(unsafe { Mmap::map(&file).expect("failed to mmap the file") });

    let mut output_dir: PathBuf = PathBuf::new();

    if args.is_extract {
        output_dir = args.output_dir.expect("output directory is not set");
    }

    let output_dir_cloned = output_dir.clone();

    if args.is_extract {
        fs::create_dir_all(output_dir).expect("could not create directory for extracting files");
    }

    let start_time = Instant::now();
    let (ssx, drx) = mpsc::channel();

    let (byte1_patterns, patterns): (Vec<Bytes>, Vec<Bytes>) = args
        .patterns
        .keys()
        .cloned()
        .into_iter()
        .partition(|x| x.len() == 1);

    let ac = AhoCorasick::new(&patterns).expect("could not initiate AhoCorasick");
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

    let byte1_ac = AhoCorasick::new(&byte1_patterns).expect("could not initiate AhoCorasick");
    let byte1_ssx_cloned = ssx.clone();
    let byte1_mmap_cloned = Arc::clone(&mmap);
    let patterns_cloned = args.patterns.clone();

    let byte1_scanner = thread::spawn(move || {
        if byte1_ac.patterns_len() > 0 {
            for c in byte1_ac.find_iter(&*byte1_mmap_cloned) {
                let pattern = &byte1_patterns[c.pattern()];

                if let Some(stream_types) = patterns_cloned.get(pattern) {
                    byte1_ssx_cloned
                        .send((c.start(), stream_types.clone()))
                        .expect("could not synchronize threads");
                }
            }
        }
    });

    drop(ssx);

    let mmap_cloned = Arc::clone(&mmap);

    let state = Arc::new(Mutex::new(State {
        silent: args.silent,
        total_streams_size: 0,
        total_streams_count: 0,
        is_extract: args.is_extract,
        processed_sectors: RangeSetBlaze::new(),
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
        for (offset, size, ext) in erx {
            extractor::extract(&mmap_cloned, offset, size, &ext, &output_dir_cloned);
        }
    });

    scanner.join().expect("scanner thread panicked");
    detector.join().expect("detector thread panicked");
    extractor.join().expect("extractor thread panicked");
    byte1_scanner.join().expect("byte1 scanner thread panicked");

    let state = state.lock().expect("could not lock the state");

    if args.is_extract && args.erase_sectors {
        let total_erased_bytes = eraser::erase_sectors(&file, &state.processed_sectors);

        if total_erased_bytes != state.total_streams_size {
            eprintln!(
                "Total erased bytes ({}) does not match the total streams size ({})",
                total_erased_bytes, state.total_streams_size
            );
        }
    }

    Summary {
        processed_bytes: mmap.len(),
        process_time: start_time.elapsed(),
        total_streams_size: state.total_streams_size,
        total_streams_count: state.total_streams_count,
    }
}

fn run_injector(args: Args) {
    let file_path = args.file_path.expect("file path is not set");
    let input_dir = args.input_dir.expect("input dir is not set");
    let input_path = input_dir.as_path();

    let dst = OpenOptions::new()
        .write(true)
        .read(true)
        .open(file_path)
        .expect("failed to open the file");

    let input_path_str = input_path
        .to_str()
        .expect("failed to get input path")
        .to_owned();

    let (sx, rx) = mpsc::channel();
    let sx_cloned = sx.clone();

    let mut mmap_dst = Arc::new(Mutex::new(None));

    if is_mmap_support(&dst) {
        mmap_dst = Arc::new(Mutex::new(Some(unsafe {
            MmapMut::map_mut(&dst).expect("failed to mmap the file")
        })));
    }

    let walker = thread::spawn(move || {
        for entry in glob(&input_path_str).unwrap() {
            if let Ok(path) = entry {
                let file_name = path.file_stem().unwrap().to_str().unwrap();
                let offset = file_name.parse::<i128>().unwrap_or(-1);

                if offset == -1 {
                    continue;
                }

                let uoffset = u64::try_from(offset).unwrap();

                sx_cloned
                    .send((path, uoffset))
                    .expect("could not synchronize threads");
            }
        }
    });

    drop(sx);

    let mmap_dst_cloned = Arc::clone(&mmap_dst);

    let injector = thread::spawn(move || {
        let mut mmap_lock = mmap_dst_cloned.lock().expect("failed to acquire lock");

        for (path, offset) in rx {
            let src = OpenOptions::new()
                .read(true)
                .open(&path)
                .expect("failed to open the file");

            let mut injected_bytes: u64 = 0;
            let mut mmap_injected = false;

            if let Some(ref mut mmap) = mmap_lock.as_mut() {
                if is_mmap_support(&src) {
                    mmap_injected = true;
                    injected_bytes = injector::inject_mmap(&src, mmap, offset as usize) as u64;
                }
            }

            if !mmap_injected {
                injected_bytes = injector::inject_io(&src, &dst, offset);
            }

            let src_size = src.metadata().unwrap().len();

            if injected_bytes != src_size {
                eprintln!(
                    "Injected bytes ({}) does not match the source file ({})",
                    injected_bytes, src_size
                );
            }

            if !args.silent {
                println!("--> Injected {} bytes @ {}", injected_bytes, offset);
            }
        }
    });

    walker.join().expect("walker thread panicked");
    injector.join().expect("injector thread panicked");

    let mut mmap_lock = mmap_dst.lock().expect("failed to acquire lock");

    if let Some(mmap) = mmap_lock.as_mut() {
        mmap.flush().expect("failed to save file on disk");
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
    println!("-> Processed: {}", humanize_size(summary.processed_bytes));
    println!("-> Process time: {:?}", summary.process_time);
    println!("-> Speed: {:.2} MB/s", speed_mbps);
    println!("-> Found streams: {}", summary.total_streams_count);
    println!(
        "-> Size of found streams: {} ({} bytes)",
        humanize_size(summary.total_streams_size),
        summary.total_streams_size
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
        silent: cli_args.silent,
        erase_sectors: cli_args.erase_sectors,
        input_dir: None,
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
        cli::Commands::Inject {
            file_path,
            input_dir,
        } => {
            println!("-> Injecting...");
            args.file_path = Some(PathBuf::from(file_path));
            args.input_dir = Some(PathBuf::from(input_dir));
            run_injector(args);
        }
    }
}
