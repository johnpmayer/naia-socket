
use clap::{App, Arg};
use hyper::{
    header::{self, HeaderValue},
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Error, Method, Response, Server, StatusCode,
};
use log::{info, warn};
use std::{
    net::{ IpAddr, SocketAddr, TcpListener },
    time::{Duration, Instant},
};
use async_trait::async_trait;
use webrtc_unreliable::{Server as RtcServer, MessageType, MessageResult, RecvError};

use futures_channel::mpsc;
use futures_core::Stream;
use futures_util::{pin_mut, select, FutureExt, SinkExt, StreamExt};
use tokio::time::{self, Interval};

use crate::server::ServerSocket;
use super::client_message::ClientMessage;
use super::client_event::ClientEvent;

const MESSAGE_BUFFER_SIZE: usize = 8;
const EVENT_BUFFER_SIZE: usize = 8;
const PERIODIC_TIMER_INTERVAL: Duration = Duration::from_secs(1);

pub struct WebrtcServerSocket {
    connect_function: Option<Box<dyn Fn(&ClientMessage) + Sync + Send>>,
    disconnect_function: Option<Box<dyn Fn(&ClientMessage) + Sync + Send>>,
    receive_function: Option<Box<dyn Fn(&ClientMessage) + Sync + Send>>,
    error_function: Option<Box<dyn Fn(&ClientMessage) + Sync + Send>>,
    message_sender: mpsc::Sender<ClientMessage>,
    message_receiver: mpsc::Receiver<ClientMessage>,
    event_sender: mpsc::Sender<ClientEvent>,
    event_receiver: mpsc::Receiver<ClientEvent>,
    periodic_timer: Interval,
}

#[async_trait]
impl ServerSocket for WebrtcServerSocket {
    fn new() -> WebrtcServerSocket {
        println!("Hello WebrtcServerSocket!");

        let (message_sender, message_receiver) = mpsc::channel(MESSAGE_BUFFER_SIZE);
        let (event_sender, event_receiver): (mpsc::Sender<ClientEvent>, mpsc::Receiver<ClientEvent>) = mpsc::channel(EVENT_BUFFER_SIZE);
        let new_server_socket = WebrtcServerSocket {
            disconnect_function: None,
            connect_function: None,
            receive_function: None,
            error_function: None,
            message_sender,
            message_receiver,
            event_sender,
            event_receiver,
            periodic_timer: time::interval(PERIODIC_TIMER_INTERVAL),
        };

        new_server_socket
    }

    async fn listen(&mut self, address: &str) {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

        let session_listen_addr: SocketAddr = address
            .parse()
            .expect("could not parse HTTP address/port");
        let webrtc_listen_ip: IpAddr = session_listen_addr.ip();
        let webrtc_listen_port = get_available_port(webrtc_listen_ip.to_string().as_str())
            .expect("no available port");
        let webrtc_listen_addr = SocketAddr::new(webrtc_listen_ip, webrtc_listen_port);

        let mut rtc_server = RtcServer::new(webrtc_listen_addr, webrtc_listen_addr)
            .await
            .expect("could not start RTC server");

        let mut cloned_event_sender_1 = self.event_sender.clone();
        let mut cloned_event_sender_2 = self.event_sender.clone();
        rtc_server.on_connection(move |socket_addr| {
            let client_message = ClientMessage {
                message: None,
                address: socket_addr
            };
            cloned_event_sender_1.send(ClientEvent::Connection(socket_addr));
        });
        rtc_server.on_disconnection(move |socket_addr| {
            let client_message = ClientMessage {
                message: None,
                address: socket_addr
            };
            cloned_event_sender_2.send(ClientEvent::Disconnection(socket_addr));
        });

        let session_endpoint = rtc_server.session_endpoint();
        let make_svc = make_service_fn(move |addr_stream: &AddrStream| {
            let session_endpoint = session_endpoint.clone();
            let remote_addr = addr_stream.remote_addr();
            async move {
                Ok::<_, Error>(service_fn(move |req| {
                    let mut session_endpoint = session_endpoint.clone();
                    async move {
                        if req.uri().path() == "/"
                            || req.uri().path() == "/index.html" && req.method() == Method::GET
                        {
                            info!("serving example index HTML to {}", remote_addr);
                            Response::builder().body(Body::from(include_str!("./echo_server.html")))
                        } else if req.uri().path() == "/new_rtc_session" && req.method() == Method::POST
                        {
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
            Server::bind(&session_listen_addr)
                .serve(make_svc)
                .await
                .expect("HTTP session server has died");
        });

        let mut message_buf = vec![0; 0x10000];

        loop {
            self.process(&mut rtc_server, &mut message_buf).await;
        }
    }

    fn on_connection(&mut self, func: impl Fn(&ClientMessage) + Sync + Send + 'static) {
        self.connect_function = Some(Box::new(func));
    }

    fn on_receive(&mut self, func: impl Fn(&ClientMessage) + Sync + Send + 'static) {
        self.receive_function = Some(Box::new(func));
    }

    fn on_error(&mut self, func: impl Fn(&ClientMessage) + Sync + Send + 'static) {
        self.error_function = Some(Box::new(func));
    }

    fn on_disconnection(&mut self, func: impl Fn(&ClientMessage) + Sync + Send + 'static) {
        self.disconnect_function = Some(Box::new(func));
    }

    fn get_sender(&mut self) -> mpsc::Sender<ClientMessage> {
        return self.message_sender.clone();
    }
}

impl WebrtcServerSocket {

    async fn process(&mut self, rtc_server: &mut RtcServer, message_buf: &mut [u8]) {

        enum Next {
            IncomingEvent(ClientEvent),
            //IncomingMessage(Result<MessageResult, RecvError>),
            OutgoingMessage(ClientMessage),
            PeriodicTimer,
        }

//                    incoming_message = rtc_server.recv(message_buf) => {
//                        Next::IncomingMessage(incoming_message)
//                    }

        let next = {

            let timer_next = self.periodic_timer.tick().fuse();
            pin_mut!(timer_next);

            select! {
                outgoing_message = self.message_receiver.next() => {
                    Next::OutgoingMessage(
                        outgoing_message.expect("message receiver closed")
                    )
                }
                incoming_event = self.event_receiver.next() => {
                    Next::IncomingEvent(
                        incoming_event.expect("message receiver closed")
                    )
                }
                _ = timer_next => {
                    Next::PeriodicTimer
                }
            }
        };

        match next {
            Next::IncomingEvent(incoming_event) => {
                match incoming_event {
                    ClientEvent::Connection(address) => {
                        let client_message = ClientMessage {
                            address,
                            message: None
                        };
                        (self.connect_function.as_ref().unwrap())(&client_message);
                    }
                    ClientEvent::Disconnection(address) => {
                        let client_message = ClientMessage {
                            address,
                            message: None
                        };
                        (self.disconnect_function.as_ref().unwrap())(&client_message);
                    }
                }
            }
//            Next::IncomingMessage(incoming_message) => {
//                match incoming_message {
//                    Ok(message_result) => {
//                        let packet_payload = &message_buf[0..message_result.message_len];
//                        let message_type = message_result.message_type;
//                        let address = message_result.remote_addr;
//
//                        let message = String::from_utf8_lossy(packet_payload);
//
//                        let client_message = ClientMessage::new(
//                            address,
//                            message.as_ref()
//                        );
//
//                        (self.receive_function.as_ref().unwrap())(&client_message);
//                    }
//                    Err(err) => {
//                        warn!("could not receive RTC message: {}", err);
//                    }
//                }
//            }
            Next::OutgoingMessage(outgoing_message) => {
                match outgoing_message.message {
                    Some(client_message) => {
                        rtc_server.send(
                            client_message.into_bytes().as_slice(),
                            MessageType::Text,
                            &outgoing_message.address
                        ).await;
                    }
                    _ => {
                        println!("What's going on?");
                    }
                }
            }
            Next::PeriodicTimer => {
                println!("tick tock ");
            }
            _ => {
                println!("How did we get here?");
            }
        }
    }
}

fn get_available_port(ip: &str) -> Option<u16> {
    (8000..9000)
        .find(|port| port_is_available(ip, *port))
}

fn port_is_available(ip: &str, port: u16) -> bool {
    match TcpListener::bind((ip, port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}