use crate::app::App;
use std::thread::sleep;

pub fn start_loop(app: &mut App) {
    loop {
        sleep(app.update_interval);
        app.update();
    }
}
