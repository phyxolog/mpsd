use memmap2::{Mmap, MmapMut};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};

pub fn is_mmap_support(file: &File) -> std::io::Result<bool> {
    let size = file.metadata()?.len();
    return Ok(!(std::mem::size_of::<usize>() < 8 && size > isize::MAX as u64));
}

pub fn inject_io(src: &File, dst: &File, offset: u64) -> std::io::Result<u64> {
    let mut reader = BufReader::new(src);
    let mut writer = BufWriter::new(dst);

    let mut buffer_size: u64 = 128 * 1024;
    let mut buffer = vec![0; buffer_size as usize];
    let mut bytes_written: u64 = 0;
    let size = src.metadata()?.len();
    let dst_size = dst.metadata()?.len();

    if size > dst_size {
        let err_msg = format!("Size {} exceeds destination size {}", size, dst_size);

        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            err_msg,
        ));
    }

    writer.seek(SeekFrom::Start(offset))?;

    while bytes_written < size {
        if bytes_written + buffer_size > size {
            buffer_size = size - bytes_written;
            buffer.resize(buffer_size as usize, 0);
        }

        reader.read_exact(&mut buffer)?;
        writer.write_all(&buffer)?;
        bytes_written += buffer_size;
    }

    return Ok(bytes_written);
}

pub fn inject_mmap(src: &File, dst: &mut MmapMut, offset: usize) -> std::io::Result<usize> {
    let mmap_src = unsafe { Mmap::map(src)? };

    let mut bytes_written = 0;
    let mut buffer_size = 128 * 1024;
    let size = src.metadata()?.len() as usize;
    let dst_size = dst.len();

    if size > dst_size {
        let err_msg = format!("Size {} exceeds destination size {}", size, dst_size);

        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            err_msg,
        ));
    }

    while bytes_written < size {
        if bytes_written + buffer_size > size {
            buffer_size = size - bytes_written;
        }

        let start = bytes_written;
        let end = start + buffer_size;

        let src = &mmap_src[start..end];
        let dst = &mut dst[offset + bytes_written..offset + bytes_written + buffer_size];

        dst.copy_from_slice(src);
        bytes_written += buffer_size;
    }

    return Ok(bytes_written);
}
