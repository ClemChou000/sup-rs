use clap::Parser;
use log::error;
use std::io::Write;
use sup_rs::{
    config::config::Config,
    controller::{client::Client, command::Command},
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

fn main() {
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .format(|buf, record| {
            writeln!(
                buf,
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

    match args.subcommand {
        Command::Start => match cli.start() {
            Ok(r) => print!("{}", r),
            Err(e) => error!("exec command start failed: {}", e),
        },
        Command::Stop => match cli.stop() {
            Ok(r) => print!("{}", r),
            Err(e) => error!("exec command stop failed: {}", e),
        },
        Command::Restart => match cli.restart() {
            Ok(r) => print!("{}", r),
            Err(e) => error!("exec command restart failed: {}", e),
        },
        Command::Kill => match cli.kill() {
            Ok(r) => print!("{}", r),
            Err(e) => error!("exec command kill failed: {}", e),
        },
        Command::Reload => match cli.reload() {
            Ok(r) => print!("{}", r),
            Err(e) => error!("exec command reload failed: {}", e),
        },
        Command::Exit => match cli.exit() {
            Ok(r) => print!("{}", r),
            Err(e) => error!("exec command exit failed: {}", e),
        },
        Command::Status => match cli.status() {
            Ok(r) => print!("{}", r),
            Err(e) => error!("exec command status failed: {}", e),
        },
        Command::Unknown => print!("unknown command"),
    }
}
