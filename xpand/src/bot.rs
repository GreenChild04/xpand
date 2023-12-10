use serenity::{model::prelude::*, async_trait, client::{Context, EventHandler}, Client};
use crate::{secrets, crypto, unwrap, log::Log};

pub struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        println!("\n\x1b[34m=== \x1b[36mClient \x1b[32mStarted\x1b[36m! Welcome to \x1b[35mXpand \x1b[33mv{} \x1b[34m===", env!("CARGO_PKG_VERSION"));
        let id = crate::segment::segment_upload("test_tmp/segment/sourcefile", 1182850912767713310, &ctx).await.unwrap();
        crate::segment::segment_download("test_tmp/segment/newfile", 1182850912767713310, id, &ctx).await.unwrap();
        assert_eq!(crypto::hash_file("test_tmp/segment/sourcefile").unwrap(), crypto::hash_file("test_tmp/segment/newfile").unwrap());
        std::process::exit(0);
    }
}

impl Bot {
    pub async fn run() {
        Log::Warning("if the client doesn't start in 10s then check your internet connection and retry".into(), None).log();
        let mut client = unwrap!(Res "while building client": Client::builder(secrets::TOKEN.trim(), GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT)
            .event_handler(Bot)
            .application_id(secrets::APP_ID)
            .await);
        unwrap!(Res "while connecting to file servers": client.start().await);
    }
}