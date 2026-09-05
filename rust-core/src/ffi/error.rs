//! FFI error codes (stable integers for Dart)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiError {
    Ok = 0,
    InvalidHandle = 1,
    Locked = 2,
    Crypto = 3,
    Network = 4,
    Storage = 5,
    InvalidPassword = 6,
    PanicWiped = 7,
    NoSession = 8,
    Serialize = 9,
    Internal = 99,
}

impl FfiError {
    pub fn code(self) -> i32 { self as i32 }
}
