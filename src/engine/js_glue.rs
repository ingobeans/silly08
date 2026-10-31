use deno_core::op2;

use crate::engine::*;

#[op2(fast)]
fn send_save_state(#[buffer(copy)] data: Vec<u8>) {
    assert!(data.len() >= SAVED_MEMORY_SIZE);
    let ptr = SAVE_STATE_DATA_PTR.lock().unwrap();
    if *ptr == 0 {
        eprintln!("SAVE_STATE_DATA_PTR is null!");
        return;
    }
    let ptr = *ptr as *mut u8;
    // std::fs::write("out.bin", &data[..SAVED_MEMORY_SIZE]).unwrap();

    // todo: assert length of pointer buffer is SAVE_STATE_SIZE

    unsafe {
        // Write header
        std::ptr::copy(SAVE_STATE_HEADER.as_ptr(), ptr, SAVE_STATE_HEADER.len());

        // Write memory data
        std::ptr::copy(
            data.as_ptr(),
            ptr.add(SAVE_STATE_HEADER.len()),
            data.len().min(SAVED_MEMORY_SIZE),
        );
    }
    SAVE_STATE_AVAILABLE.store(true, std::sync::atomic::Ordering::Relaxed);
}

#[op2(fast)]
fn send_data(#[buffer(copy)] data: Vec<u8>) {
    *SCREEN_DATA.lock().unwrap() = data;
    MESSAGE_TX
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .send(Message::ScreenData)
        .unwrap();
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
        let mut should_terminate = false;
        let mut should_save_state = false;
        let mut should_load_state = false;

        // let mut load_state = Vec::new();

        if let Ok(message) = TERMINATE_RX.lock().unwrap().as_mut().unwrap().try_recv() {
            match message {
                StatusMessage::Terminate => should_terminate = true,
                StatusMessage::SaveState => should_save_state = true,
                StatusMessage::LoadState(ptr) => {
                    should_load_state = true;
                    let ptr = ptr as *mut u8;

                    unsafe {
                        // Get header
                        let mut header = [0u8; SAVE_STATE_HEADER.len()];
                        std::ptr::copy(ptr, header.as_mut_ptr(), SAVE_STATE_HEADER.len());

                        if header != SAVE_STATE_HEADER {
                            eprintln!(
                                "Unknown save state header encountered: {}\n({header:?})",
                                String::from_utf8_lossy(&header)
                            );
                        } else {
                            // Get memory data
                            std::ptr::copy(
                                ptr.add(SAVE_STATE_HEADER.len()),
                                load_state.as_mut_ptr(),
                                SAVED_MEMORY_SIZE,
                            );
                        }
                    }
                    LOAD_STATE_AVAILABLE.store(false, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }

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
    ops = [send_data, timeout, send_save_state],
    objects = [MessagePollObject],
);
