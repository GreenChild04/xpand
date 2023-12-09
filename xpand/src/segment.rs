use std::{path::Path, io};
use base64::Engine;
use serenity::{client::Context, all::ChannelId, builder::{CreateAttachment, CreateMessage}};
use tokio::{io::AsyncReadExt, fs::File};
use crate::crypto;

/// Segments the file into 24MiB-25MiB chunks and uploads them to the server
pub async fn segment_upload(source_file: impl AsRef<Path>, channel_id: u64, ctx: &Context) -> Result<u64, io::Error> {
    let mut source_file = File::open(source_file).await?;
    let channel_id = ChannelId::from(channel_id);

    let mut i = 0u64;
    let mut buffer = vec![0u8; 24 * 1024 * 1024].into_boxed_slice(); // 24MiB buffer
    let mut segment_ids = Vec::new();

    loop {
        let bytes_read = fill_buffer(&mut buffer, &mut source_file).await?;
        println!("Read {} bytes", bytes_read);
        if bytes_read == 0 { break }; // if no bytes read then break
        
        // hash and upload segment
        let hash = crypto::hash(&buffer[0..bytes_read]);
        let attachment = CreateAttachment::bytes(&buffer[0..bytes_read], i.to_string());
        let message = CreateMessage::default()
            .content(base64::engine::general_purpose::URL_SAFE.encode(hash));
        let message = channel_id.send_files(&ctx.http, [attachment], message)
            .await
            .expect("Error uploading file");

        segment_ids.push(message.id.get());
        i += 1;
    }

    Ok(0)
}

#[inline]
async fn fill_buffer(buffer: &mut [u8], file: &mut File) -> Result<usize, io::Error> {
    let mut i = 0;
    loop {
        let bytes_read = file.read(&mut buffer[i..]).await?;
        if bytes_read == 0 { break };
        i += bytes_read;
    }

    Ok(i)
}