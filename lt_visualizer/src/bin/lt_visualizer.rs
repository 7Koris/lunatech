use lt_client::client::LunaTechClient;

fn main() {
    let client = LunaTechClient::new(3000);
    loop {
        client.get_osc_features();
     }
}
