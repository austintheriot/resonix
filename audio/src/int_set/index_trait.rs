pub trait Index {
    fn id(&self) -> usize;
}

impl Index for u8 {
    fn id(&self) -> usize {
        *self as usize
    }
}

impl Index for u16 {
    fn id(&self) -> usize {
        *self as usize
    }
}

impl Index for u32 {
    fn id(&self) -> usize {
        *self as usize
    }
}

impl Index for u64 {
    fn id(&self) -> usize {
        *self as usize
    }
}

impl Index for u128 {
    fn id(&self) -> usize {
        *self as usize
    }
}

impl Index for usize {
    fn id(&self) -> usize {
        *self
    }
}
impl Index for i8 {
    fn id(&self) -> usize {
        *self as usize
    }
}

impl Index for i16 {
    fn id(&self) -> usize {
        *self as usize
    }
}

impl Index for i32 {
    fn id(&self) -> usize {
        *self as usize
    }
}

impl Index for i64 {
    fn id(&self) -> usize {
        *self as usize
    }
}

impl Index for i128 {
    fn id(&self) -> usize {
        *self as usize
    }
}
