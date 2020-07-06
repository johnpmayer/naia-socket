use async_trait::async_trait;
use std::{net::SocketAddr, time::Duration};

use super::{
    message_sender::MessageSender,
    socket_event::SocketEvent,
    timer_handler::{TimerHandler, TimerKey},
};
use crate::error::NaiaServerSocketError;
use naia_socket_shared::Config;

/// Defines the functionality of a Naia Server Socket
#[async_trait]
pub trait ServerSocketTrait {
    /// Creates a new Server Socket, listening at a given address, and taking an
    /// optional Config
    async fn listen(socket_address: SocketAddr, config: Option<Config>) -> Self;
    /// Receive a new packet from the socket, or a tick event
    async fn receive(&mut self) -> Result<SocketEvent, NaiaServerSocketError>;
    /// Gets a MessageSender you can use to send messages through the Server
    /// Socket
    fn get_sender(&mut self) -> MessageSender;
    fn create_timer(&mut self, timer_interval: Duration) -> TimerKey;
    fn delete_timer(&mut self, key: TimerKey);
}

cfg_if! {
    if #[cfg(feature = "use-webrtc")] {
        // WebRTC Server ///
        pub use crate::webrtc_server_socket::WebrtcServerSocket;
        /// ServerSocket is an alias for a socket abstraction using either UDP or WebRTC for communications
        pub type ServerSocket = WebrtcServerSocket;
    }
    else if #[cfg(feature = "use-udp")] {
        // UDP Server
        pub use crate::udp_server_socket::UdpServerSocket;
        /// ServerSocket is an alias for a socket abstraction using either UDP or WebRTC for communications
        pub type ServerSocket = UdpServerSocket;
    }
}
