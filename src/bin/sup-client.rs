use clap::Parser;
use log::{error, info};
use std::io::Write;
use sup_rs::{
    config::config::Config,
    controller::{
        client::Client,
        command::{Command, Request},
    },
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // path to toml format config file
    #[arg(short, long)]
    config_path: String,

    #[clap(subcommand)]
    subcommand: Command,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .format(|buf, record| {
            writeln!(buf,
                "{} - {} - {} - {}",
                buf.timestamp(),
                record.file().unwrap(),
                record.level(),
                record.args()
            )
        })
        .init();

    let args = Args::parse();
    let cfg = match Config::new(&args.config_path) {
        Ok(c) => c,
        Err(e) => panic!("create config failed: {}", e.to_string()),
    };
    let cli = Client::new(cfg.sup.socket);
    match cli.request(Request::new(args.subcommand)).await {
        Ok(resp) => {
            info!("get resp: {resp}")
        }
        Err(e) => {
            error!("request failed: {e}")
        }
    }
}
