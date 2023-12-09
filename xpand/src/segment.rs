use std::{path::Path, fs::File, io::{self, BufReader, Read, Write, BufWriter}};
use serenity::client::Context;
use crate::bot::Bot;

/// Segment the file into 24-25MB smaller files
pub fn segment(source_file: impl AsRef<Path>, out_path: impl AsRef<Path>, f: impl Fn(&[u8]) -> &[u8]) -> Result<u16, io::Error> {
    let mut source_file = BufReader::new(File::open(source_file)?);
    
    let mut i = 0u16;
    let mut out_file = BufWriter::new(File::create(out_path.as_ref().join(i.to_string()))?);
    let mut buffer = vec![0u8; 4 * 1024 * 1024].into_boxed_slice(); // 4MiB buffer
    let mut current_size: usize = 0;

    loop {
        let bytes_read = source_file.read(&mut *buffer)?;
        if bytes_read == 0 { break }; // if no bytes read then break

        current_size += bytes_read; // add read size to current_size
        if current_size >= 26214400 { // if current segment larger than or equal to 25MB them move onto the next segment
            i += 1;
            current_size = bytes_read;
            out_file.flush()?;
            out_file = BufWriter::new(File::create(out_path.as_ref().join(i.to_string()))?);
        }
        
        out_file.write_all( // write segment's contents to out_file
            f(&buffer[0..bytes_read])
        )?;
    } Ok(i)
}

/// Uploads the segments to the server
pub async fn upload_segments(segments: u16, channel_id: u64, ctx: &Context, path: impl AsRef<Path>) -> Result<Box<[u64]>, io::Error> {
    let mut segment_ids = Vec::new();

    for i in 0..segments {
        let file_path = path.as_ref().join(i.to_string());
        segment_ids.push(
            Bot::upload_file(file_path, channel_id, ctx).await?
        )
    }

    Ok(segment_ids.into_boxed_slice())
}