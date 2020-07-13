use async_trait::async_trait;

use naia_socket_shared::LinkConditionerConfig;

use super::{message_sender::MessageSender, packet::Packet};
use crate::error::NaiaServerSocketError;

/// Defines the functionality of a Naia Server Socket
#[async_trait]
pub trait ServerSocketTrait: Send + Sync {
    /// Receive a new packet from the socket, or a tick event
    async fn receive(&mut self) -> Result<Packet, NaiaServerSocketError>;
    /// Gets a MessageSender you can use to send messages through the Server
    /// Socket
    fn get_sender(&mut self) -> MessageSender;
    /// Wraps the current socket in a LinkConditioner
    fn with_link_conditioner(
        self: Box<Self>,
        config: &LinkConditionerConfig,
    ) -> Box<dyn ServerSocketTrait>;
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
