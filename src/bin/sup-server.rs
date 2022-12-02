use env_logger;
use std::io::Write;
use sup_rs::controller::server::Server;

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
    let s = Server::new("./sup.sock".to_string());
    s.unwrap().run();
}
