use log::info;
use std::{net::SocketAddr, time::Duration};

use naia_client_socket::{ClientSocket, Config, MessageSender, Packet, SocketEvent};

const PING_MSG: &str = "ping";
const PONG_MSG: &str = "pong";

const SERVER_PORT: u16 = 14191;

cfg_if! {
    if #[cfg(target_arch = "wasm32")]
    {
        use std::net::IpAddr;
    }
    else
    {
        use naia_client_socket::find_my_ip_address;
    }
}

pub struct App {
    client_socket: ClientSocket,
    message_sender: MessageSender,
    message_count: u8,
    pub update_interval: Duration, // how often the app should call it's update() method
}

impl App {
    pub fn new() -> App {
        info!("Naia Client Socket Example Started");

        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let server_ip_address: IpAddr = "192.168.1.9".parse().expect("couldn't parse input IP address"); // Put your Server's IP Address here!, can't easily find this automatically from the browser
            } else {
                let server_ip_address = find_my_ip_address().expect("can't find ip address");
            }
        }

        let server_socket_address = SocketAddr::new(server_ip_address, SERVER_PORT);

        let mut client_socket =
            ClientSocket::connect(server_socket_address, Some(Config::default()));
        let mut message_sender = client_socket.get_sender();

        message_sender
            .send(Packet::new(PING_MSG.to_string().into_bytes()))
            .unwrap();

        App {
            client_socket,
            message_sender,
            message_count: 0,
            update_interval: Duration::from_millis(50),
        }
    }

    pub fn update(&mut self) {
        loop {
            match self.client_socket.receive() {
                Ok(event) => {
                    match event {
                        SocketEvent::Packet(packet) => {
                            let message = String::from_utf8_lossy(packet.payload());
                            info!("Client recv: {}", message);

                            if message.eq(PONG_MSG) && self.message_count < 10 {
                                self.message_count += 1;
                                let to_server_message: String = PING_MSG.to_string();
                                info!("Client send: {}", to_server_message);
                                self.message_sender
                                    .send(Packet::new(to_server_message.into_bytes()))
                                    .expect("send error");
                            }
                        }
                        SocketEvent::None => {
                            //info!("Client non-event");
                            return;
                        }
                    }
                }
                Err(err) => {
                    info!("Client Error: {}", err);
                }
            }
        }
    }
}
