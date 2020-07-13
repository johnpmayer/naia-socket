use async_trait::async_trait;

use naia_socket_shared::{Instant, LinkConditionerConfig, TimeQueue};

use super::{
    error::NaiaServerSocketError, message_sender::MessageSender, packet::Packet,
    server_socket::ServerSocketTrait,
};

pub struct LinkConditioner {
    config: LinkConditionerConfig,
    inner_socket: Box<dyn ServerSocketTrait>,
    result_queue: TimeQueue<Packet>,
}

impl LinkConditioner {
    pub fn new(config: &LinkConditionerConfig, socket: Box<dyn ServerSocketTrait>) -> Self {
        LinkConditioner {
            config: config.clone(),
            inner_socket: socket,
            result_queue: TimeQueue::new(),
        }
    }
}

#[async_trait]
impl ServerSocketTrait for LinkConditioner {
    async fn receive(&mut self) -> Result<Packet, NaiaServerSocketError> {
        /// TODO: Use TimeQueue
        let result = self.inner_socket.receive().await;
        result
    }

    fn get_sender(&mut self) -> MessageSender {
        self.inner_socket.get_sender()
    }

    fn with_link_conditioner(
        self: Box<Self>,
        config: &LinkConditionerConfig,
    ) -> Box<dyn ServerSocketTrait> {
        // Absolutely do not recommend decorating a socket with multiple link
        // conditioners... why would you do this??
        Box::new(LinkConditioner::new(config, self))
    }
}

impl LinkConditioner {
    fn process_packet(&mut self, packet: Packet) {
        self.result_queue.add_item(Instant::now(), packet);
    }

    fn has_packet(&self) -> bool {
        self.result_queue.has_item()
    }

    fn get_packet(&mut self) -> Packet {
        self.result_queue.pop_item().unwrap()
    }
}
