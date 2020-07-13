use crate::app::App;
use std::thread::sleep;

pub fn start_loop(app: &mut App) {
    loop {
        app.update();
    }
}
