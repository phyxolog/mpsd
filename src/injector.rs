use memmap2::{Mmap, MmapMut};
use std::fs::File;

pub fn inject(src: &File, dst: &mut MmapMut, offset: usize) -> usize {
    let mmap_src = unsafe { Mmap::map(src).expect("failed to mmap the file") };

    let mut bytes_written = 0;
    let mut buffer_size = 128 * 1024;
    let size = src.metadata().unwrap().len() as usize;

    if size > mmap_src.len() {
        return 0;
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

    return bytes_written;
}
