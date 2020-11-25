#[macro_use]
extern crate cfg_if;
extern crate log;

use std::collections::VecDeque;

use miniquad::*;

mod app;

use app::App;

struct Stage {
    ctx: Context,
    app: App,
}
impl EventHandlerFree for Stage {
    fn update(&mut self) {
        self.app.update();
        //        unsafe {
        //            naia_resend_dropped_messages();
        //
        //            if let Some(msg_queue) = &mut MESSAGE_QUEUE {
        //                if let Some(message) = msg_queue.pop_front() {
        //                    miniquad::debug!("recv: {}", &message);
        //
        //                    if MESSAGE_COUNT < 10 {
        //                        let out_msg = "ping";
        //                        miniquad::debug!("send: {}", &out_msg);
        //                        naia_send(JsObject::string(out_msg));
        //                        MESSAGE_COUNT += 1;
        //                    }
        //                }
        //            }
        //
        //            if let Some(error_queue) = &mut ERROR_QUEUE {
        //                if let Some(error) = error_queue.pop_front() {
        //                    miniquad::debug!("error: {}", &error);
        //                }
        //            }
        //        };
    }

    fn draw(&mut self) {
        self.ctx.clear(Some((0., 1., 0., 1.)), None, None);
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    let mut app = App::new();
    miniquad::start(conf::Conf::default(), |ctx| {
        UserData::free(Stage { ctx, app })
    });
}
