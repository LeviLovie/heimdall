use argh::FromArgs;

pub fn parse() -> Args {
    argh::from_env()
}

#[derive(FromArgs, Debug)]
#[argh(description = "The log analyzer")]
pub struct Args {
    #[argh(switch, short = 'd', description = "enable dry run mode")]
    pub _dry: bool,

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
