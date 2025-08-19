use clap::{Args as ClapArgs, Parser, Subcommand};

const LONG_ABOUT: &str = "Heimdall watches your code for bugs.\nSee https://github.com/LeviLovie/heimdall for more info.";

pub fn parse() -> Args {
    Args::parse()
}

#[derive(Parser, Clone, Debug)]
#[command(
    version,
    about,
    long_about = LONG_ABOUT,
)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Cmd {
    Server(ServerArgs),
    Pipe(PipeArgs),
}

#[derive(ClapArgs, Clone, Debug)]
pub struct ServerArgs {
    #[arg(long, default_value = "127.0.0.1")]
    pub address: String,

    #[arg(short, long, help = "Start a Terminal User Interface")]
    pub tui: bool,

    #[arg(long, value_name = "PORT", help = "Start a NNG server (default 62000)")]
    pub nng: Option<Option<u16>>,

    #[arg(
        long,
        value_name = "PORT",
        help = "Start a HTTP server (default 62001)"
    )]
    pub http: Option<Option<u16>>,
}

#[derive(ClapArgs, Clone, Debug)]
pub struct PipeArgs {
    #[arg(short, long, default_value = "127.0.0.1")]
    pub address: String,

    #[arg(short, long, default_value = "62000")]
    pub port: u16,

    #[arg(long)]
    pub json: bool,
}
