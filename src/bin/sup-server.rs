use clap::Parser;
use env_logger;
use std::io::Write;
use sup_rs::{config::config::Config, controller::server::Server};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // path to toml format config file
    #[arg(short, long)]
    config_path: String,
}

#[tokio::main]
fn main() {
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .format(|buf, record| {
            let ts = buf.timestamp();
            writeln!(
                buf,
                "{} - {} - {} - {}",
                ts,
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
    Server::new(cfg.sup.socket).unwrap().run().await;
}
