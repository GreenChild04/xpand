use std::{path::Path, fs::File, io::{self, BufReader, Read, Write, BufWriter}};

/// Segment the file into 24-25MB smaller files
pub fn segment(source_file: impl AsRef<Path>, out_path: impl AsRef<Path>) -> Result<(), io::Error> {
    let mut source_file = BufReader::new(File::open(source_file)?);
    
    let mut i = 0u16;
    let mut out_file = BufWriter::new(File::create(out_path.as_ref().join(i.to_string()))?);
    let mut buffer = vec![0u8; 4 * 1024 * 1024].into_boxed_slice(); // 4MiB buffer
    let mut bytes_read = source_file.read(&mut *buffer)?;
    let mut current_size: usize = 0;

    while bytes_read != 0 {
        current_size += bytes_read; // add read size to
        if current_size >= 25000000 { // if current segment larger than or equal to 25MB them move onto the next segment
            i += 1;
            current_size = bytes_read;
            out_file.flush()?;
            out_file = BufWriter::new(File::create(out_path.as_ref().join(i.to_string()))?);
        } out_file.write_all(&buffer[0..bytes_read])?; // write segment's contents to out_file
        
        bytes_read = source_file.read(&mut *buffer)?;
    }

    Ok(())
}