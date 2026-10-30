mod engine;
mod libretro;
use std::{
    ffi::c_void,
    ptr::null,
    sync::{
        Mutex,
        mpsc::{Receiver, Sender},
    },
    thread::JoinHandle,
};

use deno_core::error::AnyError;
use libretro::*;

use crate::engine::{Message, SAVE_STATE_AVAILABLE, SAVE_STATE_DATA, SCREEN_DATA, StatusMessage};

static JOIN_HANDLE: Mutex<Option<JoinHandle<Result<(), AnyError>>>> = Mutex::new(None);
static MESSAGE_RX: Mutex<Option<Receiver<Message>>> = Mutex::new(None);
static TERMINATE_TX: Mutex<Option<Sender<StatusMessage>>> = Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn retro_api_version() -> u32 {
    return RETRO_API_VERSION;
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_controller_port_device(port: u32, device: u32) {
    println!("Plugging device {} into port {}.\n", device, port);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_system_info(info: *mut retro_system_info) {
    unsafe {
        (*info).library_name = "silly08".as_ptr();
        (*info).library_version = env!("CARGO_PKG_VERSION").as_ptr();
        (*info).need_fullpath = true;
        (*info).valid_extensions = "".as_ptr();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_system_av_info(info: *mut retro_system_av_info) {
    unsafe {
        (*info).geometry.base_width = 128;
        (*info).geometry.base_height = 128;
        (*info).geometry.max_width = 128;
        (*info).geometry.max_height = 128;
        (*info).geometry.aspect_ratio = 1.0;
    }
}
fn rgba_to_xrgb8888(buf: &mut [u8]) {
    for pixel in buf.chunks_exact_mut(4) {
        // found through testing,
        // dont know why the red and blue channels are swapped but whatever
        let old_b = pixel[2];
        pixel[2] = pixel[0];
        pixel[0] = old_b;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_run() {
    let Some(cb) = *VIDEO_CB.lock().unwrap() else {
        return;
    };
    let mut data = SCREEN_DATA.lock().unwrap().clone();
    if data.len() == 0 {
        return;
    }
    unsafe {
        assert_eq!(data.len(), 128 * 128 * 4);
        // data.rotate_right(1); // rgba -> xrgb
        rgba_to_xrgb8888(&mut data);
        cb(data.as_ptr() as _, 128, 128, 4 * 128);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_load_game(info: *const retro_game_info) -> bool {
    let desc: &mut [retro_input_descriptor] = &mut [
        retro_input_descriptor {
            port: 0,
            device: RETRO_DEVICE_JOYPAD,
            index: 0,
            id: RETRO_DEVICE_ID_JOYPAD_LEFT,
            description: "Left".as_ptr(),
        },
        retro_input_descriptor {
            port: 0,
            device: RETRO_DEVICE_JOYPAD,
            index: 0,
            id: RETRO_DEVICE_ID_JOYPAD_UP,
            description: "Up".as_ptr(),
        },
        retro_input_descriptor {
            port: 0,
            device: RETRO_DEVICE_JOYPAD,
            index: 0,
            id: RETRO_DEVICE_ID_JOYPAD_DOWN,
            description: "Down".as_ptr(),
        },
        retro_input_descriptor {
            port: 0,
            device: RETRO_DEVICE_JOYPAD,
            index: 0,
            id: RETRO_DEVICE_ID_JOYPAD_RIGHT,
            description: "Right".as_ptr(),
        },
        retro_input_descriptor {
            port: 0,
            device: 0,
            index: 0,
            id: 0,
            description: 0_usize as _,
        },
    ];

    let cb = ENVIRON_CB.lock().unwrap().unwrap();
    unsafe {
        cb(RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS, desc as *mut _ as _);
    }

    let mut fmt = retro_pixel_format::RETRO_PIXEL_FORMAT_XRGB8888;
    let res = unsafe { cb(RETRO_ENVIRONMENT_SET_PIXEL_FORMAT, &mut fmt as *mut _ as _) };
    if !res {
        println!("Pixel format is not supported.");
        return false;
    }

    let mut retro_game_path = [0_u8; 4096];
    let mut index = 0;
    unsafe {
        let mut ptr = (*info).path;
        loop {
            if *ptr == 0 {
                break;
            }
            retro_game_path[index] = *ptr;
            index += 1;
            ptr = ptr.add(1);
        }
    }
    let path = String::from_utf8_lossy(&retro_game_path[..index]).to_string();
    println!("Got path: {path}");

    let Ok(data) = std::fs::read(path) else {
        return false;
    };
    let (handle, message_rx, terminate_tx) = engine::run_thread(data);
    *JOIN_HANDLE.lock().unwrap() = Some(handle);
    *MESSAGE_RX.lock().unwrap() = Some(message_rx);
    *TERMINATE_TX.lock().unwrap() = Some(terminate_tx);

    //    struct retro_audio_callback audio_cb = { audio_callback, audio_set_state };
    //    use_audio_cb = ENVIRON_CB(RETRO_ENVIRONMENT_SET_AUDIO_CALLBACK, &audio_cb);

    //    check_variables();

    //    (void)info;
    return true;
}

// callback setters

static ENVIRON_CB: Mutex<Option<EnvironmentFn>> = Mutex::new(None);
static VIDEO_CB: Mutex<Option<VideoRefreshFn>> = Mutex::new(None);
static AUDIO_CB: Mutex<Option<AudioSampleFn>> = Mutex::new(None);
static AUDIO_BATCH_CB: Mutex<Option<AudioBatchFn>> = Mutex::new(None);
static INPUT_POLL_CB: Mutex<Option<InputPollFn>> = Mutex::new(None);
static INPUT_STATE_CB: Mutex<Option<InputStateFn>> = Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_video_refresh(cb: VideoRefreshFn) {
    *VIDEO_CB.lock().unwrap() = Some(cb);
}
#[unsafe(no_mangle)]
pub extern "C" fn retro_set_audio_sample(cb: AudioSampleFn) {
    *AUDIO_CB.lock().unwrap() = Some(cb);
}
#[unsafe(no_mangle)]
pub extern "C" fn retro_set_audio_sample_batch(cb: AudioBatchFn) {
    *AUDIO_BATCH_CB.lock().unwrap() = Some(cb);
}
#[unsafe(no_mangle)]
pub extern "C" fn retro_set_input_poll(cb: InputPollFn) {
    *INPUT_POLL_CB.lock().unwrap() = Some(cb);
}
#[unsafe(no_mangle)]
pub extern "C" fn retro_set_input_state(cb: InputStateFn) {
    *INPUT_STATE_CB.lock().unwrap() = Some(cb);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_environment(cb: EnvironmentFn) {
    *ENVIRON_CB.lock().unwrap() = Some(cb);

    //    if (cb(RETRO_ENVIRONMENT_GET_LOG_INTERFACE, &logging))
    //       log_cb = logging.log;
    //    else
    //       log_cb = fallback_log;

    //    static const struct retro_controller_description controllers[] = {
    //       { "Nintendo DS", RETRO_DEVICE_SUBCLASS(RETRO_DEVICE_JOYPAD, 0) },
    //    };

    //    static const struct retro_controller_info ports[] = {
    //       { controllers, 1 },
    //       { NULL, 0 },
    //    };

    //    unsafe {
    //     cb(RETRO_ENVIRONMENT_SET_CONTROLLER_INFO, (void*)ports);
    //    }
}

// function skeletons

#[unsafe(no_mangle)]
pub extern "C" fn retro_init() {}

#[unsafe(no_mangle)]
pub extern "C" fn retro_deinit() {
    retro_unload_game();
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_unload_game() {
    let Ok(mut lock) = TERMINATE_TX.lock() else {
        return;
    };
    let Some(tx) = lock.as_mut() else { return };
    let _ = tx.send(StatusMessage::Terminate);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_serialize_size() -> usize {
    // from testing, hardcoded :sob:
    return 268435456;
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_serialize(data: *mut u8, size: usize) -> bool {
    println!("serialize called!");
    let Ok(mut lock) = TERMINATE_TX.lock() else {
        return false;
    };
    let Some(tx) = lock.as_mut() else {
        return false;
    };
    let Ok(_) = tx.send(StatusMessage::SaveState) else {
        return false;
    };

    while !SAVE_STATE_AVAILABLE.load(std::sync::atomic::Ordering::Relaxed) {}
    SAVE_STATE_AVAILABLE.store(false, std::sync::atomic::Ordering::Relaxed);

    let lock = SAVE_STATE_DATA.lock().unwrap();
    let len = lock.len().min(size);
    unsafe {
        std::ptr::copy(lock.as_ptr(), data, len);
    }
    println!("done!");

    return true;
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_unserialize(_data_: *const c_void, _size: usize) -> bool {
    return false;
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_memory_data(_id: u32) -> *const c_void {
    return null();
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_memory_size(_id: u32) -> usize {
    return 0;
}
