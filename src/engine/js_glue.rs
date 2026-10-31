use std::sync::atomic::Ordering;

use deno_core::op2;

use crate::engine::*;

#[op2(fast)]
fn send_save_state(#[buffer(copy)] data: Vec<u8>) {
    assert!(data.len() >= SAVED_MEMORY_SIZE);
    SAVE_STATE_MANAGER.write_internal_save(&data);
}

#[op2(fast)]
fn send_video_data(#[buffer(copy)] data: Vec<u8>) {
    SCREEN_DATA_MANAGER.write_frame_data(data);
}

#[repr(C)]
pub struct MessagePollObject {
    new_key_events: GcCell<bool>,
    should_terminate: GcCell<bool>,
    should_save_state: GcCell<bool>,
    should_load_state: GcCell<bool>,
    pressed_keys: GcCell<Vec<String>>,
    released_keys: GcCell<Vec<String>>,
}

#[op2]
impl MessagePollObject {
    #[getter]
    fn new_key_events(&self, isolate: &mut v8::Isolate) -> bool {
        *self.new_key_events.get(isolate)
    }
    #[getter]
    fn should_terminate(&self, isolate: &v8::Isolate) -> bool {
        *self.should_terminate.get(isolate)
    }
    #[getter]
    fn should_save_state(&self, isolate: &v8::Isolate) -> bool {
        *self.should_save_state.get(isolate)
    }
    #[getter]
    fn should_load_state(&self, isolate: &v8::Isolate) -> bool {
        *self.should_load_state.get(isolate)
    }
    #[getter]
    fn pressed_keys(&self, isolate: &v8::Isolate) -> Vec<String> {
        self.pressed_keys.get(isolate).clone()
    }
    #[getter]
    fn released_keys(&self, isolate: &v8::Isolate) -> Vec<String> {
        self.released_keys.get(isolate).clone()
    }

    #[static_method]
    #[cppgc]
    fn poll(#[buffer] load_state: &mut [u8]) -> MessagePollObject {
        let should_terminate = SHOULD_TERMINATE.load(Ordering::Relaxed);
        let should_save_state = SAVE_STATE_MANAGER.is_save_requested();
        let should_load_state = SAVE_STATE_MANAGER.load_internal_save_if_requested(load_state);

        let events = KEY_STATE.flush_events();

        MessagePollObject {
            should_terminate: GcCell::new(should_terminate),
            should_save_state: GcCell::new(should_save_state),
            new_key_events: GcCell::new(events.new_key_events),
            pressed_keys: GcCell::new(events.pressed_keys),
            released_keys: GcCell::new(events.released_keys),
            should_load_state: GcCell::new(should_load_state),
        }
    }
}

unsafe impl GarbageCollected for MessagePollObject {
    fn trace(&self, _visitor: &mut v8::cppgc::Visitor) {}
    fn get_name(&self) -> &'static std::ffi::CStr {
        c"MessagePollObject"
    }
}

#[op2(async(lazy), fast)]
async fn timeout(#[bigint] time: u64) {
    tokio::time::sleep(Duration::from_millis(time)).await;
}

extension!(
    my_ext,
    ops = [send_video_data, timeout, send_save_state],
    objects = [MessagePollObject],
);
