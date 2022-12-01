use sup_rs::controller::server::Server;

fn main() {
    let s = Server::new("./sup.sock".to_string());
    s.unwrap().run();
}
