use serenity::{model::prelude::*, async_trait, client::{Context, EventHandler}, Client};
use crate::secrets;

pub struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, _ctx: Context, _ready: Ready) {
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // msg.channel_id.send_files(&ctx.http, );
        if msg.content == "!ping" {
            let _ = msg.channel_id.say(&ctx.http, "Pong!").await;
        } else if msg.content.starts_with("!echo ") {
            let _ = msg.channel_id.say(&ctx.http, format!("> {}", msg.content.strip_prefix("!echo ").unwrap_or("<echo error>"))).await;
        }
    }
}

impl Bot {
    pub async fn run() {
        let mut client = Client::builder(secrets::TOKEN.trim(), GatewayIntents::all())
            .event_handler(Bot)
            .application_id(secrets::APP_ID)
            .await
            .expect("Error creating client");
        client.start().await.unwrap();
    }
}