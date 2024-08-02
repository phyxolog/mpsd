use std::fs::OpenOptions;
use std::io::Write;
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

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&output_path)
        .expect("failed to create a file");

    let mut bytes_written = 0;
    let mut buffer_size = 128 * 1024;

    while bytes_written < size {
        if bytes_written + buffer_size > size {
            buffer_size = size - bytes_written;
        }

        let start = offset + bytes_written;
        let end = start + buffer_size;

        file.write_all(&buffer[start..end])
            .expect("failed to write in file");

        bytes_written += buffer_size;
    }

    return bytes_written;
}
