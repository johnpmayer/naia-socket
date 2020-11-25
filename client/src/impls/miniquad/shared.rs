use std::{collections::VecDeque, net::SocketAddr};

pub static mut MESSAGE_QUEUE: Option<VecDeque<String>> = None;
pub static mut ERROR_QUEUE: Option<VecDeque<String>> = None;

#[no_mangle]
extern "C" {
    pub fn naia_connect(server_socket_address: JsObject);
    pub fn naia_send(message: JsObject);
    pub fn naia_resend_dropped_messages();
    pub fn naia_create_string(buf: *const u8, max_len: u32) -> JsObject;
    pub fn naia_free_object(js_object: JsObjectWeak);
    pub fn naia_unwrap_to_str(js_object: JsObjectWeak, buf: *mut u8, max_len: u32);
    pub fn naia_string_length(js_object: JsObjectWeak) -> u32;
}

#[repr(transparent)]
pub struct JsObject(u32);

impl JsObject {
    pub fn weak(&self) -> JsObjectWeak {
        JsObjectWeak(self.0)
    }
}
#[derive(Clone, Copy)]
#[repr(transparent)]
struct JsObjectWeak(u32);

impl Drop for JsObject {
    fn drop(&mut self) {
        unsafe {
            naia_free_object(self.weak());
        }
    }
}

impl JsObject {
    pub fn string(string: &str) -> JsObject {
        unsafe { naia_create_string(string.as_ptr() as _, string.len() as _) }
    }

    pub fn to_string(&self, buf: &mut String) {
        let len = unsafe { naia_string_length(self.weak()) };

        if len as usize > buf.len() {
            buf.reserve(len as usize - buf.len());
        }
        unsafe { buf.as_mut_vec().set_len(len as usize) };
        unsafe { naia_unwrap_to_str(self.weak(), buf.as_mut_vec().as_mut_ptr(), len as u32) };
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
