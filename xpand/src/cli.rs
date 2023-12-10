use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {

}

#[derive(Subcommand)]
pub enum Command {
    #[command(about="A mere test command")]
    Test,
}