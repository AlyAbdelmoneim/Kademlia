use clap::*;

#[derive(Debug, Clone, Parser)]
#[clap(name = "kade")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Commands {
    ///store a key value pair
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
}
