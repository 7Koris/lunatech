use lt_client::client::Client;

use std::{fmt::format, net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket}, str::FromStr};

fn main() {
    let client = Client::new(3000);
    loop {
       client.get_osc_features();
    }
}