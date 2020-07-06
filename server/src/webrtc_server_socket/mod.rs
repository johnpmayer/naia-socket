use async_trait::async_trait;
use hyper::{
    header::{self, HeaderValue},
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Error as HyperError, Method, Response, Server, StatusCode,
};
use log::info;
use std::net::{IpAddr, SocketAddr, TcpListener};
use webrtc_unreliable::{
    MessageResult, MessageType, RecvError, SendError, Server as InnerRtcServer, SessionEndpoint,
};

use futures_channel::mpsc;
use futures_util::{pin_mut, select, FutureExt, StreamExt};
use tokio::time::{self, Interval};

use super::{message_sender::MessageSender, socket_event::SocketEvent};
use crate::{error::NaiaServerSocketError, Packet, ServerSocketTrait};
use naia_socket_shared::Config;

const CLIENT_CHANNEL_SIZE: usize = 8;

#[derive(Debug)]
pub struct WebrtcServerSocket {
    rtc_server: RtcServer,
    to_client_sender: mpsc::Sender<Packet>,
    to_client_receiver: mpsc::Receiver<Packet>,
    tick_timer: Interval,
    receive_buffer: Vec<u8>,
}

#[async_trait]
impl ServerSocketTrait for WebrtcServerSocket {
    async fn listen(socket_address: SocketAddr, config: Option<Config>) -> WebrtcServerSocket {
        let webrtc_listen_ip: IpAddr = socket_address.ip();
        let webrtc_listen_port =
            get_available_port(webrtc_listen_ip.to_string().as_str()).expect("no available port");
        let webrtc_listen_addr = SocketAddr::new(webrtc_listen_ip, webrtc_listen_port);

        let (to_client_sender, to_client_receiver) = mpsc::channel(CLIENT_CHANNEL_SIZE);

        let rtc_server = RtcServer::new(webrtc_listen_addr).await;

        let tick_interval = match config {
            Some(config) => config.tick_interval,
            None => Config::default().tick_interval,
        };

        let socket = WebrtcServerSocket {
            rtc_server,
            to_client_sender,
            to_client_receiver,
            tick_timer: time::interval(tick_interval),
            receive_buffer: vec![0; 0x10000], /* Hopefully get rid of this one day.. next version
                                               * of webrtc-unreliable should make that happen */
        };

        let session_endpoint = socket.rtc_server.session_endpoint();
        let make_svc = make_service_fn(move |addr_stream: &AddrStream| {
            let session_endpoint = session_endpoint.clone();
            let remote_addr = addr_stream.remote_addr();
            async move {
                Ok::<_, HyperError>(service_fn(move |req| {
                    let mut session_endpoint = session_endpoint.clone();
                    async move {
                        if req.uri().path() == "/new_rtc_session" && req.method() == Method::POST {
                            info!("WebRTC session request from {}", remote_addr);
                            match session_endpoint.http_session_request(req.into_body()).await {
                                Ok(mut resp) => {
                                    resp.headers_mut().insert(
                                        header::ACCESS_CONTROL_ALLOW_ORIGIN,
                                        HeaderValue::from_static("*"),
                                    );
                                    Ok(resp.map(Body::from))
                                }
                                Err(err) => Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!("error: {}", err))),
                            }
                        } else {
                            Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Body::from("not found"))
                        }
                    }
                }))
            }
        });

        tokio::spawn(async move {
            Server::bind(&socket_address)
                .serve(make_svc)
                .await
                .expect("HTTP session server has died");
        });

        socket
    }

    async fn receive(&mut self) -> Result<SocketEvent, NaiaServerSocketError> {
        enum Next {
            FromClientMessage(Result<MessageResult, RecvError>),
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
                let rtc_server = &mut self.rtc_server;
                let from_client_message_receiver_next = rtc_server.recv(receive_buffer).fuse();
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
                    Ok(message_result) => {
                        let address = message_result.remote_addr;
                        let payload: Vec<u8> = self.receive_buffer[0..message_result.message_len]
                            .iter()
                            .cloned()
                            .collect();
                        return Ok(SocketEvent::Packet(Packet::new_raw(
                            address,
                            payload.into_boxed_slice(),
                        )));
                    }
                    Err(err) => {
                        return Err(NaiaServerSocketError::Wrapped(Box::new(err)));
                    }
                },
                Next::ToClientMessage(packet) => {
                    let address = packet.address();

                    match self
                        .rtc_server
                        .send(packet.payload(), MessageType::Binary, &address)
                        .await
                    {
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

    fn get_sender(&mut self) -> MessageSender {
        return MessageSender::new(self.to_client_sender.clone());
    }
}

fn get_available_port(ip: &str) -> Option<u16> {
    (8000..9000).find(|port| port_is_available(ip, *port))
}

fn port_is_available(ip: &str, port: u16) -> bool {
    match TcpListener::bind((ip, port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

struct RtcServer {
    inner: InnerRtcServer,
}

impl RtcServer {
    pub async fn new(address: SocketAddr) -> RtcServer {
        let inner = InnerRtcServer::new(address, address)
            .await
            .expect("could not start RTC server");

        return RtcServer { inner };
    }

    pub fn session_endpoint(&self) -> SessionEndpoint {
        self.inner.session_endpoint()
    }

    pub async fn recv(&mut self, buf: &mut [u8]) -> Result<MessageResult, RecvError> {
        self.inner.recv(buf).await
    }

    pub async fn send(
        &mut self,
        message: &[u8],
        message_type: MessageType,
        remote_addr: &SocketAddr,
    ) -> Result<(), SendError> {
        self.inner.send(message, message_type, remote_addr).await
    }
}

use std::fmt;
impl fmt::Debug for RtcServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RtcServer")
    }
}
