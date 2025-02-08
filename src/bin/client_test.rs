use lunatech::client::Client;
use lunatech::service;

use std::{fmt::format, net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket}, str::FromStr};
const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 69);
const MULTICAST_PORT: u16 = 3000;
fn main() {
    let client = Client::new();
    loop {
       client.get_osc_features();
    }
}