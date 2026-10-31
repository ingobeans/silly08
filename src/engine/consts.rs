/// The entire size of the heap memory of the runtime.
/// Found from testing, hardcoded :sob:
pub const MEMORY_SIZE: usize = 268435456;

/// The size of the memory saved in a save state. Arbitrary value.
/// From testing it seems that only about ~9% of the heap is used normally,
/// so for performance and storage reasons we only keep a smaller portion of the full memory.
/// (in this case 12.5%, giving a bit of headroom past the standard 9%)
pub const SAVED_MEMORY_SIZE: usize = MEMORY_SIZE / 8;

pub const SAVE_STATE_HEADER: &[u8] = b"silly08-00";

/// The total size of a save state, containing both the saved memory and the header.
pub const SAVE_STATE_SIZE: usize = SAVED_MEMORY_SIZE + SAVE_STATE_HEADER.len();

pub const SCREEN_DATA_SIZE: usize = 128 * 128 * 4;
