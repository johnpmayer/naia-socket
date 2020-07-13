use rand::Rng;

use super::{
    instant::Instant, link_conditioner_config::LinkConditionerConfig, time_queue::TimeQueue,
};

/// Given a config object which describes the network conditions to be
/// simulated, process an incoming packet, adding it to a TimeQueue at the
/// correct timestamp
pub fn process_packet<T: Eq>(
    config: &LinkConditionerConfig,
    time_queue: &mut TimeQueue<T>,
    packet: T,
) {
    let mut rng = rand::thread_rng();
    if rng.gen_range(0.0, 1.0) <= config.incoming_loss {
        // drop the packet
        println!("link conditioner: packet lost");
        return;
    }
    if rng.gen_range(0.0, 1.0) <= config.incoming_corruption {
        //TODO: corrupt the packet
        println!("link conditioner: packet corrupted");
        return;
    }
    let mut latency: u32 = config.incoming_latency;
    if rng.gen_bool(0.5) {
        latency += rng.gen_range(0, config.incoming_jitter);
    } else {
        latency -= rng.gen_range(0, config.incoming_jitter);
    }
    let mut packet_timestamp = Instant::now();
    packet_timestamp.add_millis(latency);
    time_queue.add_item(packet_timestamp, packet);
}
