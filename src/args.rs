use argh::FromArgs;

pub fn parse() -> Args {
    argh::from_env()
}

#[derive(FromArgs, Debug)]
#[argh(description = "The log analyzer")]
pub struct Args {
    #[argh(switch, short = 'd', description = "enable dry run mode")]
    pub dry: bool,

    #[argh(subcommand)]
    pub cmd: Cmd,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Cmd {
    Receive(ReceiveArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "receive",
    description = "Receive logs from a specified address and port"
)]
pub struct ReceiveArgs {
    #[argh(
        option,
        short = 'a',
        description = "address to bind the socket to",
        default = "String::from(\"127.0.0.1\")"
    )]
    pub address: String,

    #[argh(
        option,
        short = 'p',
        description = "port to bind the socket to",
        default = "62000"
    )]
    pub port: u16,
}
