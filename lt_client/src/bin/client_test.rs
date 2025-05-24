use lt_client::client::LunaTechClient;

fn main() {
    let client = LunaTechClient::new(3000);
    loop {
        println!("{}", client.audio_features.rms.get());
    }
}
