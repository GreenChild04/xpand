use std::{thread::sleep, time::Duration};
use xpand::{secrets, bot::Bot, loading_bar::LoadingBar};
use serenity::{Client, model::gateway::GatewayIntents};

// #[tokio::main]
// async fn main() {
//     let mut client = Client::builder(secrets::TOKEN.trim(), GatewayIntents::all())
//         .event_handler(Bot)
//         .application_id(secrets::APP_ID)
//         .await
//         .expect("Error creating client");
//     client.start().await.unwrap();
// }

fn main() {
    let mut loading_bar = LoadingBar::new(1024);
    while !loading_bar.update(0.05, 4 * 1024 * 1024).draw("downloading file") {
        sleep(Duration::from_millis(50));
    }
}