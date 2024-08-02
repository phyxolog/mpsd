use memmap2::MmapMut;
use range_set_blaze::RangeSetBlaze;
use std::fs::File;

pub fn erase_sectors(file: &File, sectors: &RangeSetBlaze<usize>) -> usize {
    let mut mmap = unsafe { MmapMut::map_mut(file).expect("failed to mmap the file") };

    let mut total_bytes_written = 0;
    let mut buffer_size = 128 * 1024;

    for r in sectors.ranges() {
        let mut bytes_written = 0;
        let size = r.end() - r.start();
        let start: usize = r.start().clone();

        while bytes_written < size {
            if bytes_written + buffer_size > size {
                buffer_size = size - bytes_written;
            }

            mmap[start + bytes_written..start + bytes_written + buffer_size].fill(0);
            bytes_written += buffer_size;
        }

        total_bytes_written += bytes_written;
    }

    mmap.flush().expect("failed to save file on disk");

    return total_bytes_written;
}
