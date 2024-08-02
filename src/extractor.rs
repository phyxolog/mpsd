use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::path::PathBuf;

pub fn extract(
    buffer: &[u8],
    offset: usize,
    size: usize,
    ext: &str,
    output_dir: &PathBuf,
) -> usize {
    let file_name = format!("{}_{}.{}", offset, size, ext);
    let output_path = output_dir.as_path().join(file_name);

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&output_path)
        .expect("failed to create a file");

    file.set_len(size as u64)
        .expect("failed to set len for the file");

    let mut mmap = unsafe { MmapMut::map_mut(&file).expect("failed to mmap the file") };

    let mut bytes_written = 0;
    let mut buffer_size = 128 * 1024;

    while bytes_written < size {
        if bytes_written + buffer_size > size {
            buffer_size = size - bytes_written;
        }

        let start = offset + bytes_written;
        let end = start + buffer_size;
        let dst = &mut mmap[bytes_written..bytes_written + buffer_size];

        dst.copy_from_slice(&buffer[start..end]);

        bytes_written += buffer_size;
    }

    mmap.flush().expect("failed to save file on disk");

    return bytes_written;
}
