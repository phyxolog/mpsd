use aho_corasick::AhoCorasick;
use bytes::Bytes;
use detector::{Detector, StreamType};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Instant;

use memmap2::Mmap;

mod detector;

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
            println!(
                "Found {:?} stream @ {:#08x} ({} bytes)",
                media_type, offset, size
            )
        }
        _ => {}
    }
}

fn main() {
    let path = env::args()
        .nth(1)
        .expect("supply a single path as the program argument");

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

    let duration = start_time.elapsed();
    let processed_bytes = mmap.len() as f64;
    let elapsed_seconds = duration.as_secs_f64();
    let speed_mbps = (processed_bytes / (1024.0 * 1024.0)) / elapsed_seconds;

    println!("Processing completed in {:?}", start_time.elapsed());
    println!("Processing speed: {:.2} MB/s", speed_mbps);
}
