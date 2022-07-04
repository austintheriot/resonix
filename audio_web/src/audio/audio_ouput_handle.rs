use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Clone, Debug)]
/// Stores the most recent frame data for visualization purposes
pub struct AudioOutputHandle {
    /// How many frames to consider when calculating a weighted moving average
    num_frames: usize,
    /// This is the number of channels in each frame (i.e. the length of each frame vector)
    num_channels: usize,
    prev_frames: Arc<Mutex<VecDeque<Vec<f32>>>>,
    uuid: Uuid,
}

impl AudioOutputHandle {
    pub const NUM_FRAMES_DEFAULT: usize = 20;
    pub const NUM_CHANNELS_DEFAULT: usize = 2;

    /// Replaces all previous frames with a new frame
    /// This is necessary when the number of channels in a frame change
    fn recalibrate_num_channels(&mut self, new_frame: &Vec<f32>) {
        let mut prev_frames_lock = self.prev_frames.lock().unwrap();
        let new_num_channels = new_frame.len();

        // adjust num_channels to match new new_frame
        self.num_channels = new_num_channels;

        // fill the prev_frame buffer with the new_frame
        prev_frames_lock.clear();
        for _ in 0..self.num_frames {
            prev_frames_lock.push_back(new_frame.clone());
        }
    }

    pub fn add_frame(&mut self, frame: Vec<f32>) {
        // adjust frames if length doesn't match
        if frame.len() != self.num_channels {
            self.recalibrate_num_channels(&frame)
        } else {
            let mut prev_frames_lock = self.prev_frames.lock().unwrap();
            prev_frames_lock.pop_front();
            prev_frames_lock.push_back(frame);
        }
    }

    pub fn get_simple_moving_average(&self) -> Vec<f32> {
        // this is a moving average of each channel's amplitude across the previous stored frames
        let mut moving_average = Vec::from(vec![0.0; self.num_channels]);
        let prev_frames_lock = self.prev_frames.lock().unwrap();

        // get sum for each channel
        for frame in prev_frames_lock.iter() {
            for (channel_index, channel) in frame.iter().enumerate() {
                moving_average[channel_index] += channel.abs();
            }
        }

        let prev_frames_len = prev_frames_lock.len() as f32;
        // get average for each channel
        for channel in moving_average.iter_mut() {
            *channel /= prev_frames_len;
        }

        moving_average
    }
}

impl Default for AudioOutputHandle {
    fn default() -> Self {
        let mut default_prev_frames = VecDeque::with_capacity(Self::NUM_FRAMES_DEFAULT);
        for _ in 0..Self::NUM_FRAMES_DEFAULT {
            let default_frame = vec![0.0; Self::NUM_CHANNELS_DEFAULT];
            default_prev_frames.push_back(default_frame);
        }

        Self {
            num_channels: Self::NUM_CHANNELS_DEFAULT,
            num_frames: Self::NUM_FRAMES_DEFAULT,
            prev_frames: Arc::new(Mutex::new(default_prev_frames)),
            uuid: Uuid::new_v4(),
        }
    }
}

impl PartialEq for AudioOutputHandle {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}
