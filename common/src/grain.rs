
/// Contains information about where in a buffer the grain should sample from
#[derive(Clone,Copy,Debug)]
pub struct Grain {
    pub start_frame: usize,
    pub end_frame: usize,
    pub current_frame: usize,
    pub finished: bool,
    /// the number of frames between `start_frame` and `end_frame`
    pub len: usize,
}

impl Default for Grain {
    fn default() -> Self {
        Self {
            start_frame: 0,
            current_frame: 0,
            end_frame: 0,
            finished: true,
            len: 0,
        }
    }
}

impl Grain {
    pub fn new(start_frame: usize, end_frame: usize) -> Self {
        debug_assert!(start_frame < end_frame);
        Grain {
            start_frame,
            current_frame: start_frame,
            end_frame,
            finished: false,
            len: end_frame - start_frame,
        }
    }

    /// Increments the current_frame and returns it.
    /// 
    /// If the grain is already finished, this is a no-op and `None` is returned.
    pub fn get_next_frame(&mut self) -> Option<usize> {
        if self.finished {
            return None;
        }

        self.current_frame += 1;
        if self.current_frame == self.end_frame {
            self.finished = true;
        }

        Some(self.current_frame)
    }
}