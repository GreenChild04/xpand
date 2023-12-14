// use std::{thread::sleep, time::Duration};
use clap::Parser;
use xpand::cli::Cli;

#[tokio::main]
async fn main() {
    Cli::parse().execute().await;
}

// fn main() {
//     let mut loading_bar = LoadingBar::new(1024);
//     while !loading_bar.update(0.05, 4 * 1024 * 1024).draw("downloading file") {
//         sleep(Duration::from_millis(50));
//     }
// }