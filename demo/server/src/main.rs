#[macro_use]
extern crate log;

use std::net::SocketAddr;

use simple_logger;
use smol::io;

use naia_server_socket::{find_my_ip_address, LinkConditionerConfig, Packet, ServerSocket};

const SERVER_PORT: u16 = 14191;
const PING_MSG: &str = "ping";
const PONG_MSG: &str = "pong";

fn main() -> io::Result<()> {
    smol::block_on(async {
        simple_logger::init_with_level(log::Level::Info).expect("A logger was already initialized");

        info!("Naia Server Socket Example Started");

        let current_ip_address = find_my_ip_address().expect("can't find ip address");
        let current_socket_address = SocketAddr::new(current_ip_address, SERVER_PORT);

        let mut server_socket = ServerSocket::listen(current_socket_address)
            .await
            .with_link_conditioner(&LinkConditionerConfig::good_condition());

        let mut sender = server_socket.get_sender();

        loop {
            match server_socket.receive().await {
                Ok(packet) => {
                    let address = packet.address();
                    let message = String::from_utf8_lossy(packet.payload());
                    info!("Server recv <- {}: {}", address, message);

                    if message.eq(PING_MSG) {
                        let to_client_message: String = PONG_MSG.to_string();
                        info!("Server send -> {}: {}", address, to_client_message);
                        sender
                            .send(Packet::new(address, to_client_message.into_bytes()))
                            .await
                            .expect("send error");
                    }
                }
                Err(error) => {
                    info!("Server Error: {}", error);
                }
            }
        }
    })
}
