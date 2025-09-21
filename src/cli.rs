use clap::*;

#[derive(Debug, Clone, Parser)]
#[clap(name = "kade")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Commands {
    Init {
        #[clap(short, long)]
        name: String,

        #[clap(short, long)]
        port: Option<u16>,

        #[clap(long)]
        bootstrap_ip: Option<String>,

        #[clap(long)]
        bootstrap_port: Option<u16>,
    },
}
