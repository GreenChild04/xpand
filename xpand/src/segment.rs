use std::{path::Path, io};
use base64::Engine;
use serenity::{client::Context, all::{ChannelId, MessageId}, builder::{CreateAttachment, CreateMessage}};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, fs::File};
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
        if bytes_read == 0 { break }; // if no bytes read then break
        
        // hash and upload segment and add it to the list of segment ids
        segment_ids.push(upload_bytes(&buffer[..bytes_read], &format!("segment_{}", i), &channel_id, ctx)
            .await
            .expect("Error uploading segment"));
        i += 1;
    }

    Ok(upload_bytes(
        &bincode::serialize(&segment_ids.into_boxed_slice()).expect("Error serializing segment ids"),
        "file_map",
        &channel_id,
        ctx
    ).await.expect("Error uploading segment ids"))
}

/// Downloads the segments from the server and recontructs the original file
pub async fn segment_download(file_path: impl AsRef<Path>, channel_id: u64, fmap_id: u64, ctx: &Context) -> Result<(), io::Error> {
    let channel_id = ChannelId::from(channel_id);
    let file_map = MessageId::from(fmap_id);
    let mut file = File::create(file_path).await?;

    let segment_ids: Vec<u64> = bincode::deserialize(
            &download_bytes(&file_map, &channel_id, ctx)
            .await
            .expect("Error downloading segment ids"))
        .expect("Error deserializing segment ids");

    for segment_id in segment_ids {
        let segment = download_bytes(&MessageId::from(segment_id), &channel_id, ctx)
            .await
            .expect("Error downloading segment");
        file.write_all(&segment).await?;
    }

    Ok(())
}

/// Fills the buffer with data from the file
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

/// Uploads bytes to the server
#[inline]
async fn upload_bytes(bytes: &[u8], file_name: &str, channel_id: &ChannelId, ctx: &Context) -> Result<u64, serenity::prelude::SerenityError> {
    let attachment = CreateAttachment::bytes(bytes, file_name);
    let message = CreateMessage::default()
        .content(base64::engine::general_purpose::URL_SAFE.encode(crypto::hash(bytes)));
    let message = channel_id.send_files(&ctx.http, [attachment], message).await?;

    Ok(message.id.get())
}

/// Downloads bytes from the server
#[inline]
async fn download_bytes(message_id: &MessageId, channel_id: &ChannelId, ctx: &Context) -> Result<Box<[u8]>, serenity::prelude::SerenityError> {
    let message = channel_id.message(&ctx.http, message_id).await?;
    let expected_hash = base64::engine::general_purpose::URL_SAFE
        .decode(message.content.as_bytes())
        .expect("Error decoding hash");
    let attachment = message.attachments.first().expect("Error getting attachment");
    let data = attachment.download().await?.into_boxed_slice();

    let hash = crypto::hash(&data);
    if expected_hash != hash {
        panic!("Hashes do not match");
    }

    Ok(data)
}