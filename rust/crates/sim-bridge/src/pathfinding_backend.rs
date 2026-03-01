use std::sync::atomic::{AtomicU8, Ordering};

pub const PATHFIND_BACKEND_AUTO: u8 = 0;
pub const PATHFIND_BACKEND_CPU: u8 = 1;
pub const PATHFIND_BACKEND_GPU: u8 = 2;

static PATHFIND_BACKEND_MODE: AtomicU8 = AtomicU8::new(PATHFIND_BACKEND_AUTO);

#[inline]
pub fn set_backend_mode(mode: u8) {
    PATHFIND_BACKEND_MODE.store(mode, Ordering::Relaxed);
}

#[inline]
pub fn get_backend_mode() -> u8 {
    PATHFIND_BACKEND_MODE.load(Ordering::Relaxed)
}

#[inline]
pub fn parse_backend_mode(mode: &str) -> Option<u8> {
    match mode.to_ascii_lowercase().as_str() {
        "auto" => Some(PATHFIND_BACKEND_AUTO),
        "cpu" => Some(PATHFIND_BACKEND_CPU),
        "gpu" => Some(PATHFIND_BACKEND_GPU),
        _ => None,
    }
}

#[inline]
pub fn backend_mode_to_str(mode: u8) -> &'static str {
    match mode {
        PATHFIND_BACKEND_CPU => "cpu",
        PATHFIND_BACKEND_GPU => "gpu",
        _ => "auto",
    }
}

#[inline]
pub fn resolve_backend_mode_code(mode: u8) -> u8 {
    match mode {
        PATHFIND_BACKEND_CPU => PATHFIND_BACKEND_CPU,
        PATHFIND_BACKEND_GPU => {
            if cfg!(feature = "gpu") {
                PATHFIND_BACKEND_GPU
            } else {
                PATHFIND_BACKEND_CPU
            }
        }
        _ => {
            if cfg!(feature = "gpu") {
                PATHFIND_BACKEND_GPU
            } else {
                PATHFIND_BACKEND_CPU
            }
        }
    }
}

#[inline]
pub fn resolve_backend_mode_str(mode: u8) -> &'static str {
    match resolve_backend_mode_code(mode) {
        PATHFIND_BACKEND_GPU => "gpu",
        _ => "cpu",
    }
}

#[inline]
pub fn has_gpu_backend() -> bool {
    cfg!(feature = "gpu")
}
