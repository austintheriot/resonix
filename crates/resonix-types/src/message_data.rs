use std::any::Any;

pub enum MessageData {
    U8(u8),
    F32(f32),
    STRING(String),
    BYTES(Box<[u8]>),
    ANY(Box<dyn Any>),
    // Add more message types
}
