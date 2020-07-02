use futures_channel::mpsc;
use futures_util::{pin_mut, select, FutureExt, StreamExt};
use std::{io::Error as IoError, net::SocketAddr};
use tokio::{
    net::UdpSocket,
    time::{self, Interval},
};

use naia_socket_shared::Config;

use crate::{error::NaiaServerSocketError, Packet};

use super::{message_sender::MessageSender, socket_event::SocketEvent};

const CLIENT_CHANNEL_SIZE: usize = 8;

#[derive(Debug)]
pub struct UdpServerSocket {
    socket: UdpSocket,
    to_client_sender: mpsc::Sender<Packet>,
    to_client_receiver: mpsc::Receiver<Packet>,
    tick_timer: Interval,
    receive_buffer: Vec<u8>,
}

impl UdpServerSocket {
    pub async fn listen(socket_address: SocketAddr, config: Option<Config>) -> UdpServerSocket {
        let socket = UdpSocket::bind(socket_address).await.unwrap();

        let tick_interval = match config {
            Some(config) => config.tick_interval,
            None => Config::default().tick_interval,
        };

        let (to_client_sender, to_client_receiver) = mpsc::channel(CLIENT_CHANNEL_SIZE);

        UdpServerSocket {
            socket,
            to_client_sender,
            to_client_receiver,
            tick_timer: time::interval(tick_interval),
            receive_buffer: vec![0; 0x10000], /* Hopefully get rid of this one day.. next version
                                               * of webrtc-unreliable should make that happen */
        }
    }

    pub async fn receive(&mut self) -> Result<SocketEvent, NaiaServerSocketError> {
        enum Next {
            FromClientMessage(Result<(usize, SocketAddr), IoError>),
            ToClientMessage(Packet),
            PeriodicTimer,
        }

        loop {
            let next = {
                let timer_next = self.tick_timer.tick().fuse();
                pin_mut!(timer_next);

                let to_client_receiver_next = self.to_client_receiver.next().fuse();
                pin_mut!(to_client_receiver_next);

                let receive_buffer = &mut self.receive_buffer;
                let udp_socket = &mut self.socket;
                let from_client_message_receiver_next = udp_socket.recv_from(receive_buffer).fuse();
                pin_mut!(from_client_message_receiver_next);

                select! {
                    from_client_result = from_client_message_receiver_next => {
                        Next::FromClientMessage(from_client_result)
                    }
                    to_client_message = to_client_receiver_next => {
                        Next::ToClientMessage(
                            to_client_message.expect("to server message receiver closed")
                        )
                    }
                    _ = timer_next => {
                        Next::PeriodicTimer
                    }
                }
            };

            match next {
                Next::FromClientMessage(from_client_message) => match from_client_message {
                    Ok((message_len, message_address)) => {
                        let payload: Vec<u8> = self.receive_buffer[0..message_len]
                            .iter()
                            .cloned()
                            .collect();
                        return Ok(SocketEvent::Packet(Packet::new_raw(
                            message_address,
                            payload.into_boxed_slice(),
                        )));
                    }
                    Err(err) => {
                        return Err(NaiaServerSocketError::Wrapped(Box::new(err)));
                    }
                },
                Next::ToClientMessage(packet) => {
                    let address = packet.address();

                    match self.socket.send_to(packet.payload(), &address).await {
                        Err(_) => {
                            return Err(NaiaServerSocketError::SendError(address));
                        }
                        _ => {}
                    }
                }
                Next::PeriodicTimer => {
                    return Ok(SocketEvent::Tick);
                }
            }
        }
    }

    pub fn get_sender(&mut self) -> MessageSender {
        return MessageSender::new(self.to_client_sender.clone());
    }
}
