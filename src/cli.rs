use clap::*;

#[derive(Debug, Clone, Parser)]
#[clap(name = "kade")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Commands {
    Add {
        /// the key to store
        #[clap(short, long)]
        key: String,

        /// the value to store
        #[clap(short, long)]
        value: String,
    },

    Init {
        #[clap(short, long)]
        name: String,

        #[clap(short, long)]
        port: u16,
    },

    Ping {
        #[clap(short, long)]
        address: String,
    },
}
