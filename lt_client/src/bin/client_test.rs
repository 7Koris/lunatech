

use std::thread;

use lt_client::client::LunaTechClient;


fn main() {
    let client = LunaTechClient::new(3000);
    client.start_client();
    thread::park();
}