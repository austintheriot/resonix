use std::collections::VecDeque;

use super::audio_output_action::AudioOutputAction;

#[derive(Clone, Debug, PartialEq)]
/// Stores the most recent frame data for visualization purposes
pub struct AudioOutput {
    /// How many frames to consider when calculating a weighted moving average
    num_frames: usize,
    /// This is the number of channels in each frame (i.e. the length of each frame vector)
    num_channels: usize,
    prev_frames: VecDeque<Vec<f32>>,
}

impl AudioOutput {
    /// Replaces all previous frames with a new frame
    /// This is necessary when the number of channels in a frame change
    fn recalibrate_num_channels(&mut self, new_frame: &Vec<f32>) {
        let new_num_channels = new_frame.len();

        // adjust num_channels to match new new_frame
        self.num_channels = new_num_channels;

        // fill the prev_frame buffer with the new_frame
        self.prev_frames.clear();
        for _ in 0..self.num_frames {
            self.prev_frames.push_back(new_frame.clone());
        }
    }
}

impl AudioOutputAction for AudioOutput {
    const NUM_FRAMES_DEFAULT: usize = 2;
    const NUM_CHANNELS_DEFAULT: usize = 2;

    fn add_frame(&mut self, frame: Vec<f32>) {
        // adjust frames if length doesn't match
        if frame.len() != self.num_channels {
            self.recalibrate_num_channels(&frame)
        } else {
            self.prev_frames.pop_front();
            self.prev_frames.push_back(frame);
        }
    }

    fn get_simple_moving_average(&self) -> Vec<f32> {
        // this is a moving average of each channel's amplitude across the previous stored frames
        let mut moving_average = vec![0.0; self.num_channels];

        // get sum for each channel
        for frame in self.prev_frames.iter() {
            for (channel_index, channel) in frame.iter().enumerate() {
                moving_average[channel_index] += channel.abs();
            }
        }

        let prev_frames_len = self.prev_frames.len() as f32;
        // get average for each channel
        for channel in moving_average.iter_mut() {
            *channel /= prev_frames_len;
        }

        moving_average
    }
}

impl Default for AudioOutput {
    fn default() -> Self {
        let mut default_prev_frames = VecDeque::with_capacity(Self::NUM_FRAMES_DEFAULT);
        for _ in 0..Self::NUM_FRAMES_DEFAULT {
            let default_frame = vec![0.0; Self::NUM_CHANNELS_DEFAULT];
            default_prev_frames.push_back(default_frame);
        }

        Self {
            num_channels: Self::NUM_CHANNELS_DEFAULT,
            num_frames: Self::NUM_FRAMES_DEFAULT,
            prev_frames: default_prev_frames,
        }
    }
}
