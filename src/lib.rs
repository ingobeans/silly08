#![allow(clippy::missing_safety_doc)]
mod engine;
mod libretro;
use std::{
    ffi::c_void,
    ptr::null,
    sync::{Mutex, atomic::Ordering},
    thread::JoinHandle,
};

use deno_core::error::AnyError;
use libretro::*;

use crate::engine::*;

static JOIN_HANDLE: Mutex<Option<JoinHandle<Result<(), AnyError>>>> = Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn retro_api_version() -> u32 {
    RETRO_API_VERSION
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_controller_port_device(port: u32, device: u32) {
    println!("Plugging device {} into port {}.\n", device, port);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_get_system_info(info: *mut retro_system_info) {
    unsafe {
        (*info).library_name = "silly08".as_ptr();
        (*info).library_version = env!("CARGO_PKG_VERSION").as_ptr();
        (*info).need_fullpath = true;
        (*info).valid_extensions = "".as_ptr();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_get_system_av_info(info: *mut retro_system_av_info) {
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
        // Found through testing,
        // I don't know why the red and blue channels are swapped but whatever
        pixel.swap(2, 0);
    }
}

static KEY_MAP: Mutex<[bool; BUTTONS.len()]> = Mutex::new([false; _]);

#[unsafe(no_mangle)]
pub extern "C" fn retro_run() {
    let input_poll_cb = INPUT_POLL_CB
        .lock()
        .unwrap()
        .expect("Can't run without input_poll_cb set.");
    let video_cb = VIDEO_CB
        .lock()
        .unwrap()
        .expect("Can't run without video_cb set.");
    let input_state_cb = INPUT_STATE_CB
        .lock()
        .unwrap()
        .expect("Can't run without input_state_cb set.");

    unsafe {
        input_poll_cb();
    }

    let mut key_map_lock = KEY_MAP.lock().unwrap();

    let mut state = KEY_STATE.update_lock();
    for (index, (button, _, key)) in BUTTONS.iter().enumerate() {
        let key_state = unsafe { input_state_cb(0, RETRO_DEVICE_JOYPAD, 0, *button) } == 1;

        if key_state && !key_map_lock[index] {
            // Key became pressed down
            state.pressed_keys.push(key.to_string());
        } else if !key_state && key_map_lock[index] {
            // Key was released
            state.released_keys.push(key.to_string());
        }

        key_map_lock[index] = key_state;
    }

    if let Some(mut data) = SCREEN_DATA_MANAGER.get_new_frame() {
        unsafe {
            assert_eq!(data.len(), SCREEN_DATA_SIZE);
            rgba_to_xrgb8888(&mut data);
            video_cb(data.as_ptr() as _, 128, 128, 4 * 128);

            // Not sure if this is needed, but I want to ensure the mutex lock is held until
            // video_cb finishes.
            drop(data)
        }
    }
}

pub static BUTTONS: &[(u32, &str, &str)] = &[
    (RETRO_DEVICE_ID_JOYPAD_LEFT, "Left", "Left"),
    (RETRO_DEVICE_ID_JOYPAD_UP, "Up", "Up"),
    (RETRO_DEVICE_ID_JOYPAD_DOWN, "Down", "Down"),
    (RETRO_DEVICE_ID_JOYPAD_RIGHT, "Right", "Right"),
    (RETRO_DEVICE_ID_JOYPAD_A, "Button O", "Z"),
    (RETRO_DEVICE_ID_JOYPAD_B, "Button X", "X"),
    (RETRO_DEVICE_ID_JOYPAD_START, "Button Pause", "Enter"),
];

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_load_game(info: *const retro_game_info) -> bool {
    // Convert BUTTONS to list with input descriptors (and an additional null entry at the end)
    let mut input_descriptors: [_; BUTTONS.len() + 1] = std::array::from_fn(|i| {
        if i == BUTTONS.len() {
            retro_input_descriptor::null()
        } else {
            retro_input_descriptor {
                port: 0,
                device: RETRO_DEVICE_JOYPAD,
                index: 0,
                id: BUTTONS[i].0,
                description: BUTTONS[i].1.as_ptr(),
            }
        }
    });

    let cb = ENVIRON_CB.lock().unwrap().unwrap();
    unsafe {
        cb(
            RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS,
            &mut input_descriptors as *mut _ as _,
        );
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
    let handle = run_thread(data);
    *JOIN_HANDLE.lock().unwrap() = Some(handle);

    true
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_deinit() {
    retro_unload_game();
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_unload_game() {
    SHOULD_TERMINATE.store(true, Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_serialize_size() -> usize {
    SAVE_STATE_SIZE
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_serialize(data: *mut u8, size: usize) -> bool {
    if size != SAVE_STATE_SIZE {
        return false;
    }

    SAVE_STATE_MANAGER.get_save_blocking(data);
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_unserialize(data: *const u8, size: usize) -> bool {
    if size != SAVE_STATE_SIZE {
        return false;
    }

    SAVE_STATE_MANAGER.load_save_blocking(data);
    true
}

// Callback setters:

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
}

// Function skeletons:

#[unsafe(no_mangle)]
pub extern "C" fn retro_init() {}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_memory_data(_id: u32) -> *const c_void {
    null()
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_memory_size(_id: u32) -> usize {
    0
}
