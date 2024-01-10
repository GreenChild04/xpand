use std::{path::Path, io, time::Instant};
use base64::Engine;
use serenity::{client::Context, all::{ChannelId, MessageId}, builder::{CreateAttachment, CreateMessage}};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, fs::File};
use crate::{crypto, unwrap, log::{Log, self}, mapper::{Mapper, MapperType}, loading_bar::LoadingBar};

/// Segments the file into 24MiB-25MiB chunks and uploads them to the server
pub async fn segment_upload(source_file_path: impl AsRef<Path>, password: Option<String>, channel_id: u64, ctx: &Context) -> Result<u64, io::Error> {
    let mut source_file = File::open(&source_file_path).await?;
    let channel_id = ChannelId::from(channel_id);

    let mut loading_bar = LoadingBar::new(source_file.metadata().await?.len() as f64 / 1024.0 / 1024.0);
    loading_bar.draw("uploading file segments (this may take a while)");

    let mut i = 0u64;
    let mut buffer = vec![0u8; 24 * 1024 * 1024].into_boxed_slice(); // 24MiB buffer
    let mut segment_ids = Vec::new();

    loop {
        let time = Instant::now();
        let bytes_read = fill_buffer(&mut buffer, &mut source_file).await?;
        if bytes_read == 0 { break }; // if no bytes read then break
        Log::Info(format!("read the bytes of segment_{i} from file `{}`", source_file_path.as_ref().to_string_lossy()), None).log();

        // hash and upload segment and add it to the list of segment ids
        segment_ids.push(match &password {
            Some(passwd) => {
                unwrap!(Res "while uploading file segment": upload_bytes(
                    unwrap!(Opt ("program was unable to encrypt file segment") "while encrypting file segment": &crypto::encrypt(passwd.trim(), &buffer[..bytes_read])),
                    &format!("segment_{}", i),
                    &channel_id,
                    ctx,
                ).await)
            },
            None => {
                unwrap!(Res "while uploading file segment": upload_bytes(
                    &buffer[..bytes_read],
                    &format!("segment_{}", i),
                    &channel_id,
                    ctx,
                ).await)
            },
        }); i += 1;

        // update loading bar
        loading_bar
            .update(time.elapsed().as_secs_f32(), bytes_read as u32)
            .draw("uploading file segments");
    }

    Ok(unwrap!(Res "while uploading mapper": upload_bytes(
        unwrap!(Res "while serializing mapper": &bincode::serialize(
            &Mapper::new(
                match &password {
                    Some(passwd) => MapperType::EncryptedFile(
                        source_file.metadata().await?.len(),
                        Box::new(crypto::hash_password(passwd.trim()))
                    ),
                    None => MapperType::File(source_file.metadata().await?.len()),
                },
                segment_ids.into_boxed_slice()
            ),
        )),
        "mapper",
        &channel_id,
        ctx
    ).await))
}

/// Downloads the segments from the server and recontructs the original file
pub async fn segment_download(file_path: impl AsRef<Path>, channel_id: u64, fmap_id: u64, ctx: &Context) -> Result<(), io::Error> {
    let channel_id = ChannelId::from(channel_id);
    let file_map = MessageId::from(fmap_id);
    let mut file = File::create(&file_path).await?;
    
    Log::Info("loading mapper...".into(), None).log();
    let mapper: Mapper = unwrap!(Res "while loading downloaded mapper (mapper is corrupted)": bincode::deserialize(
        unwrap!(Res "while downloaded mapper": &download_bytes(&file_map, &channel_id, ctx).await)
    )); Log::Info("successfully loaded mapper".into(), None).log();

    let file_size = match &mapper.mtype {
        MapperType::File(x) => *x,
        MapperType::EncryptedFile(x, _) => *x,
    };

    let password = match &mapper.mtype {
        MapperType::File(_) => None,
        MapperType::EncryptedFile(_, passwd) => {
            // ask for the password
            Log::Warning("file is encrypted".into(), Some("enter the password below to decrypt the file".into())).log();
            Some(log::ask_password(&passwd))
        },
    };

    let mut loading_bar = LoadingBar::new(file_size as f64 / 1024.0 / 1024.0);
    loading_bar.draw("downloading file segments (this may take a while)");
    
    mapper.verify_version(); // version safety check
    Log::Info("mapper version verified".into(), None).log();

    for (i, segment_id) in mapper.ids.iter().enumerate() {
        let time = Instant::now();

        Log::Info(format!("downloading segment_{}...", i), None).log();
        let segment = unwrap!(Res "while downloading file segment": download_bytes(&MessageId::from(*segment_id), &channel_id, ctx).await);
        let segment_len = segment.len();
        Log::Info(format!("successfully downloaded segment_{}", i), None).log();
        file.write_all(&match &mapper.mtype {
            MapperType::File(_) => segment,
            MapperType::EncryptedFile(..) => unwrap!(Opt ("program was unable to decrypt file segment") "downloaded file segment is likely to be corrupted": crypto::decrypt(password.as_ref().unwrap(), &segment)),
        }).await?;
        Log::Info(format!("successfully wrote segment_{} to file", i), None).log();

        loading_bar
            .update(time.elapsed().as_secs_f32(), segment_len as u32)
            .draw("downloading file segments");
    } file.flush().await?;

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
    let expected_hash = unwrap!(Res "while decoding file hash (file segment or mapper corrupted)": base64::engine::general_purpose::URL_SAFE.decode(message.content.as_bytes()));
    let attachment = unwrap!(Opt ("invalid file id in mapper") "while pulling file from server": message.attachments.first());
    let data = attachment.download().await?.into_boxed_slice();

    let hash = crypto::hash(&data);
    if expected_hash != hash {
        Log::Error("downloaded file is different from the file that was uploaded".into(), Some("file is corrupted".into()));
    } Ok(data)
}
