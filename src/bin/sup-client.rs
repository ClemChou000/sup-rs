use sup_rs::controller::client::Client;
fn main() {
    let c = Client::new("./sup.sock".to_string());
    let resp = c.start();
    print!("resp: {:?}", resp)
}
