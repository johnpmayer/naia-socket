use std::{
    future::Future,
    io::Error as IoError,
    net::{IpAddr, SocketAddr, TcpListener},
    panic::catch_unwind,
    thread,
};

use async_executor::{Executor, Task};
use async_io::block_on;
use futures_lite::future;
use once_cell::sync::Lazy;

use async_trait::async_trait;
use hyper::{
    header::{self, HeaderValue},
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Error as HyperError, Method, Response, Server, StatusCode,
};
use log::info;
use webrtc_unreliable::{
    MessageResult, MessageType, SendError, Server as InnerRtcServer, SessionEndpoint,
};

use futures_channel::mpsc;
use futures_util::{pin_mut, select, FutureExt, StreamExt};

use naia_socket_shared::LinkConditionerConfig;

use super::{link_conditioner::LinkConditioner, message_sender::MessageSender};
use crate::{error::NaiaServerSocketError, Packet, ServerSocketTrait};

const CLIENT_CHANNEL_SIZE: usize = 8;

#[derive(Debug)]
pub struct WebrtcServerSocket {
    rtc_server: RtcServer,
    to_client_sender: mpsc::Sender<Packet>,
    to_client_receiver: mpsc::Receiver<Packet>,
}

impl WebrtcServerSocket {
    pub async fn listen(socket_address: SocketAddr) -> Box<dyn ServerSocketTrait> {
        let webrtc_listen_ip: IpAddr = socket_address.ip();
        let webrtc_listen_port =
            get_available_port(webrtc_listen_ip.to_string().as_str()).expect("no available port");
        let webrtc_listen_addr = SocketAddr::new(webrtc_listen_ip, webrtc_listen_port);

        let (to_client_sender, to_client_receiver) = mpsc::channel(CLIENT_CHANNEL_SIZE);

        let rtc_server = RtcServer::new(webrtc_listen_addr).await;

        let socket = WebrtcServerSocket {
            rtc_server,
            to_client_sender,
            to_client_receiver,
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

        //gotta spawn this into a separate thread w/o tokio
        Server::bind(&socket_address).serve(make_svc).await;

        Box::new(socket)
    }
}

#[async_trait]
impl ServerSocketTrait for WebrtcServerSocket {
    async fn receive(&mut self) -> Result<Packet, NaiaServerSocketError> {
        enum Next {
            FromClientMessage(Result<Packet, IoError>),
            ToClientMessage(Packet),
        }

        loop {
            let next = {
                let to_client_receiver_next = self.to_client_receiver.next().fuse();
                pin_mut!(to_client_receiver_next);

                let rtc_server = &mut self.rtc_server;
                let from_client_message_receiver_next = rtc_server.recv().fuse();
                pin_mut!(from_client_message_receiver_next);

                select! {
                    from_client_result = from_client_message_receiver_next => {
                        Next::FromClientMessage(
                            match from_client_result {
                                Ok(msg) => {
                                    Ok(Packet::new(msg.remote_addr, msg.message.as_ref().to_vec()))
                                }
                                Err(err) => { Err(err) }
                            }
                        )
                    }
                    to_client_message = to_client_receiver_next => {
                        Next::ToClientMessage(
                            to_client_message.expect("to server message receiver closed")
                        )
                    }
                }
            };

            match next {
                Next::FromClientMessage(from_client_message) => match from_client_message {
                    Ok(packet) => {
                        return Ok(packet);
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
            }
        }
    }

    fn get_sender(&mut self) -> MessageSender {
        return MessageSender::new(self.to_client_sender.clone());
    }

    fn with_link_conditioner(
        self: Box<Self>,
        config: &LinkConditionerConfig,
    ) -> Box<dyn ServerSocketTrait> {
        Box::new(LinkConditioner::new(config, self))
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

    pub async fn recv(&mut self) -> Result<MessageResult<'_>, IoError> {
        self.inner.recv().await
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
