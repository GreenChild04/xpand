use std::io;
use base64::Engine;
use serenity::{model::prelude::*, async_trait, client::{Context, EventHandler}, Client};
use crate::{secrets, crypto};

pub struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        println!("Bot is ready");
        let id = crate::segment::segment_upload("test_tmp/segment/sourcefile", 1182850912767713310, &ctx).await.unwrap();
        crate::segment::segment_download("test_tmp/segment/newfile", 1182850912767713310, id, &ctx).await.unwrap();
        assert_eq!(crypto::hash_file("test_tmp/segment/sourcefile").unwrap(), crypto::hash_file("test_tmp/segment/newfile").unwrap());
        std::process::exit(0);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            let _ = msg.channel_id.say(&ctx.http, "Pong!").await;
        } else if msg.content.starts_with("!echo ") {
            let _ = msg.channel_id.say(&ctx.http, format!("> {}", msg.content.strip_prefix("!echo ").unwrap_or("<echo error>"))).await;
        }
    }
}

impl Bot {
    pub async fn run() {
        let mut client = Client::builder(secrets::TOKEN.trim(), GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT)
            .event_handler(Bot)
            .application_id(secrets::APP_ID)
            .await
            .expect("Error creating client");
        client.start().await.unwrap();
    }

    /// Downloads a file from the server
    #[inline]
    pub async fn download_file(message_id: u64, channel_id: u64, ctx: &Context) -> Result<Box<[u8]>, io::Error> {
        let channel_id = ChannelId::from(channel_id);
        let message_id = MessageId::from(message_id);
        let message = channel_id.message(&ctx.http, message_id)
            .await
            .expect("Error getting message");

        let expected_hash = base64::engine::general_purpose::URL_SAFE.decode(message.content.as_bytes()).expect("Error decoding hash");
        let attachment = message.attachments.first().expect("Error getting attachment");
        let data = attachment.download().await.expect("Error downloading file").into_boxed_slice();
        
        let hash = crypto::hash(&data);
        if expected_hash != hash {
            panic!("Hashes do not match");
        }

        Ok(data)
    }
}