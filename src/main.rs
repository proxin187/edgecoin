mod blockchain;
mod network;
mod tui;

use network::Network;

use clap::{Parser, Subcommand};


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// formatted as `{addr}:{port}`
    #[arg(long, action)]
    host: String,

    /// connect to a already running network, this takes the address and port of any node formatted
    /// the same way as the host argument
    #[arg(long, action)]
    connect: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();


    let mut tui = tui::Tui::new()?;

    tui.enter()?;

    while !tui.should_close() {
        tui.draw()?;
    }

    tui.exit()?;

    /*
    let mut network = Network::new(args.host)?;

    if let Some(connect) = args.connect {
        network.connect(connect)?;
    }

    network.run()?;
    */

    Ok(())
}


