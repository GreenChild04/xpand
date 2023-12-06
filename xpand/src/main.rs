use xpand::{secrets, bot::Bot};
use serenity::{Client, model::gateway::GatewayIntents};

#[tokio::main]
async fn main() {
    let mut client = Client::builder(secrets::TOKEN.trim(), GatewayIntents::all())
        .event_handler(Bot)
        .application_id(secrets::APP_ID)
        .await
        .expect("Error creating client");
    client.start().await.unwrap();
}