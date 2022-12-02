use clap::Parser;
use env_logger;
use std::io::Write;
use sup_rs::controller::client::Client;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // path to toml format config file
    #[arg(short, long)]
    config_path: String,
}

fn main() {
    let args = Args::parse();

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
    let c = Client::new("./sup.sock".to_string());
    let resp = c.start();
    print!("resp: {:?}", resp)
}
