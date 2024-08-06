use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::path::PathBuf;

pub fn extract(
    buffer: &[u8],
    offset: usize,
    size: usize,
    ext: &str,
    output_dir: &PathBuf,
) -> std::io::Result<usize> {
    let file_name = format!("{}.{}", offset, ext);
    let output_path = output_dir.as_path().join(file_name);

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&output_path)?;

    file.set_len(size as u64)?;

    let mut mmap = unsafe { MmapMut::map_mut(&file)? };

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

    mmap.flush()?;

    return Ok(bytes_written);
}
