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
    if gen_range_f32(0.0, 1.0) <= config.incoming_loss {
        // drop the packet
        println!("link conditioner: packet lost");
        return;
    }
    if gen_range_f32(0.0, 1.0) <= config.incoming_corruption {
        //TODO: corrupt the packet
        println!("link conditioner: packet corrupted");
        return;
    }
    let mut latency: u32 = config.incoming_latency;
    if config.incoming_jitter > 0 {
        if gen_bool() {
            latency += gen_range_u32(0, config.incoming_jitter);
        } else {
            latency -= gen_range_u32(0, config.incoming_jitter);
        }
    }
    let mut packet_timestamp = Instant::now();
    packet_timestamp.add_millis(latency);
    time_queue.add_item(packet_timestamp, packet);
}

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        // Wasm //
        use js_sys::Math::random;
        fn gen_range_f32(lower: f32, upper: f32) -> f32 {
            let rand_range: f32 = random() as f32 * (upper - lower);
            return rand_range + lower;
        }
        fn gen_range_u32(lower: u32, upper: u32) -> u32 {
            let rand_range: u32 = (random() * f64::from(upper - lower)) as u32;
            return rand_range + lower;
        }
        fn gen_bool() -> bool {
            return random() < 0.5;
        }

    } else {
        // Linux //
        use rand::Rng;
        fn gen_range_f32(lower: f32, upper: f32) -> f32 {
            return rand::thread_rng().gen_range(lower, upper);
        }
        fn gen_range_u32(lower: u32, upper: u32) -> u32 {
            return rand::thread_rng().gen_range(lower, upper);
        }
        fn gen_bool() -> bool {
            return rand::thread_rng().gen_bool(0.5);
        }
    }
}
