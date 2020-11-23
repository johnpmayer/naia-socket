use std::collections::VecDeque;

use miniquad::*;

static mut MESSAGE_COUNT: u8 = 0;
static mut MESSAGE_QUEUE: Option<VecDeque<String>> = None;
static mut ERROR_QUEUE: Option<VecDeque<String>> = None;

#[repr(transparent)]
pub struct JsObject(u32);

impl JsObject {
    pub fn weak(&self) -> JsObjectWeak {
        JsObjectWeak(self.0)
    }
}
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct JsObjectWeak(u32);

impl Drop for JsObject {
    fn drop(&mut self) {
        unsafe {
            js_free_object(self.weak());
        }
    }
}

impl JsObject {
    pub fn string(string: &str) -> JsObject {
        unsafe { js_create_string(string.as_ptr() as _, string.len() as _) }
    }

    pub fn to_string(&self, buf: &mut String) {
        let len = unsafe { js_string_length(self.weak()) };

        if len as usize > buf.len() {
            buf.reserve(len as usize - buf.len());
        }
        unsafe { buf.as_mut_vec().set_len(len as usize) };
        unsafe { js_unwrap_to_str(self.weak(), buf.as_mut_vec().as_mut_ptr(), len as u32) };
    }
}

#[no_mangle]
pub extern "C" fn receive(message: JsObject) {
    let mut message_string = String::new();

    message.to_string(&mut message_string);

    unsafe {
        if let Some(msg_queue) = &mut MESSAGE_QUEUE {
            msg_queue.push_back(message_string);
        }
    }
}

#[no_mangle]
pub extern "C" fn error(error: JsObject) {
    let mut error_string = String::new();

    error.to_string(&mut error_string);

    unsafe {
        if let Some(error_queue) = &mut ERROR_QUEUE {
            error_queue.push_back(error_string);
        }
    }
}

#[no_mangle]
extern "C" {
    fn connect(server_socket_address: JsObject);
    fn send(message: JsObject);
    fn resend_dropped_messages();
    fn js_create_string(buf: *const u8, max_len: u32) -> JsObject;
    fn js_free_object(js_object: JsObjectWeak);
    fn js_unwrap_to_str(js_object: JsObjectWeak, buf: *mut u8, max_len: u32);
    fn js_string_length(js_object: JsObjectWeak) -> u32;
}

struct Stage {
    ctx: Context,
}
impl EventHandlerFree for Stage {
    fn update(&mut self) {
        unsafe {
            resend_dropped_messages();

            if let Some(msg_queue) = &mut MESSAGE_QUEUE {
                if let Some(message) = msg_queue.pop_front() {
                    miniquad::debug!("recv: {}", &message);

                    if MESSAGE_COUNT < 10 {
                        let out_msg = "ping";
                        miniquad::debug!("send: {}", &out_msg);
                        send(JsObject::string(out_msg));
                        MESSAGE_COUNT += 1;
                    }
                }
            }

            if let Some(error_queue) = &mut ERROR_QUEUE {
                if let Some(error) = error_queue.pop_front() {
                    miniquad::debug!("error: {}", &error);
                }
            }
        };
    }

    fn draw(&mut self) {
        self.ctx.clear(Some((0., 1., 0., 1.)), None, None);
    }
}

fn main() {
    unsafe {
        MESSAGE_QUEUE = Some(VecDeque::new());
        ERROR_QUEUE = Some(VecDeque::new());
        connect(JsObject::string("192.168.86.38:14191"));
        send(JsObject::string("ping"));
    }
    miniquad::start(conf::Conf::default(), |ctx| UserData::free(Stage { ctx }));
}
