use base64::Engine;
use clap::{Parser, Subcommand};
use serenity::{all::{Ready, GatewayIntents}, client::{EventHandler, Context}, async_trait, Client};

use crate::{log::Log, unwrap, secrets};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(short, long, help="If you want the program to log everything")]
    verbose: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about="A mere test command")]
    Test,
    #[command(about="Uploads a file to the server")]
    Upload {
        #[arg(index=1, help="The path of the file you want to upload")]
        file: String,
    },
    #[command(about="Downloads a file from the server")]
    Download {
        #[arg(short, long, help="The path the file will be downloaded as")]
        path: String,
        #[arg(index=1, help="The b64 link to the file (eg 'MTIzNDU2Nzg')")]
        link: String,
    },
}

#[async_trait]
impl EventHandler for Cli {
    async fn ready(&self, ctx: Context, _: Ready) {
        // Ready message
        println!("\n\x1b[34m=== \x1b[36mClient \x1b[32mStarted\x1b[36m! Welcome to \x1b[35mXpand \x1b[33mv{} \x1b[34m===", env!("CARGO_PKG_VERSION"));

        unsafe { crate::log::VERBOSE = self.verbose };

        // Match the subcommand
        use Command as C;
        match &self.command {
            C::Test => println!("Hello, World!"),
            C::Upload { file } => {
                let id = unwrap!(Res "while uploading file": crate::segment::segment_upload(
                    file, 
                    secrets::CHANNEL_ID,
                    &ctx
                ).await);
                Log::Success(format!("successfully uploaded file `{file}`"), None).log();
                let id_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&id.to_be_bytes());
                Log::Success(id_b64.clone(), Some(format!("to download the file use this command: `xpand download {}`", id_b64))).log();
            },
            C::Download { path, link } => {
                let id = u64::from_be_bytes(unwrap!(Res "while decoding link": base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(link.as_bytes())).try_into().unwrap_or_default());
                unwrap!(Res "while downloading file": crate::segment::segment_download(
                    path,
                    secrets::CHANNEL_ID,
                    id,
                    &ctx
                ).await);
                Log::Success(format!("successfully downloaded file `{}`", path), None).log();
            },
        } std::process::exit(0);
    }
}

impl Cli {
    #[inline]
    pub async fn execute(self) {
        Log::Warning("if the client doesn't start in 10s then check your internet connection and retry".into(), None).log();
        let mut client = unwrap!(Res "while building client": Client::builder(secrets::TOKEN.trim(), GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT)
            .event_handler(self)
            .application_id(secrets::APP_ID)
            .await);
        unwrap!(Res "while connecting to file servers": client.start().await);
    }
}