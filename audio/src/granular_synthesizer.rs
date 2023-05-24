use crate::grain::Grain;
use crate::granular_synthesizer_action::GranularSynthesizerAction;
use crate::max::Max;
use crate::min::Min;
use crate::percentage::Percentage;
use crate::{Envelope, Index, IntSet, NumChannels, SINE_ENVELOPE};
use log::info;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;
use std::time::Duration;

/// Accepts a reference to a buffer of Vec<f32> audio sample data.
///
/// Generates random multi-channel audio grain output.
pub struct GranularSynthesizer {
    /// How many channels of sound to generate per frame
    num_channels: NumChannels,

    /// Sample rate of the surrounding context
    sample_rate: u32,

    /// External audio buffer that this GranularSynthesizer should read grains from
    buffer: Arc<Vec<f32>>,

    /// used to generate random indexes
    rng: StdRng,

    grain_len: Duration,

    /// The minimum index that samples can be taken from,
    /// ranging from 0.0 -> 1.0 (i.e. Duration of the buffer)
    selection_start: Percentage,

    /// The maximum index that samples can be taken from,
    /// ranging from 0.0 -> 1.0 (i.e. percentage of the buffer)
    selection_end: Percentage,

    /// This is a counter that gets incremented on every frame.
    /// This allows performing some actions (such as replacing grains) to occur
    /// only so often.
    refresh_counter: u32,

    /// This determines the interval (in samples) at which too-long grains are marked `finished`
    /// and `finished` grains are replaced with new ones.
    ///
    /// A higher interval produces a slower transition to new selected regions.
    ///
    /// A lower interval produces a faster transition to new selected regions.
    ///
    /// It is preferred for this interval to be a prime number to minimize the amount of
    /// sample overlap, where one grain is exactly in-sync with another, producing a unified
    /// and/or chorus effect (or exaggerated amplification).
    refresh_interval: u32,

    /// List of grains and their current progress through the buffer.
    ///
    /// 1 array element = 1 grain = 1 channel of audio
    fresh_grains: IntSet<Grain>,

    /// List of grains and their current progress through the buffer.
    ///
    /// 1 array element = 1 grain = 1 channel of audio
    finished_grains: IntSet<Grain>,

    /// Volume envelope used for controlling the volume of each grain's playback
    envelope: Envelope<f32>,

    /// Internally used to track which grains extend past the buffer
    /// after the buffer selection is updated. Storing in an internal Vec
    /// prevents allocating on the audio render loop
    finished_grain_indexes: Vec<usize>,
}

impl GranularSynthesizerAction for GranularSynthesizer {
    fn new() -> Self {
        let default_buffer = Arc::new(Vec::new());
        let fresh_grains = IntSet::with_capacity(Self::DEFAULT_NUM_CHANNELS);
        let mut finished_grains = IntSet::with_capacity(Self::DEFAULT_NUM_CHANNELS);
        finished_grains.extend((0..Self::DEFAULT_NUM_CHANNELS).map(|i| Self::new_grain(i as u32)));

        Self {
            sample_rate: Self::DEFAULT_SAMPLE_RATE,
            buffer: default_buffer,
            rng: rand::rngs::StdRng::from_entropy(),
            grain_len: Self::DEFAULT_GRAIN_LEN,
            selection_start: Percentage::from(0.0),
            selection_end: Percentage::from(1.0),
            num_channels: NumChannels::new(Self::DEFAULT_NUM_CHANNELS),
            refresh_counter: 0,
            refresh_interval: Self::DEFAULT_REFRESH_INTERVAL,
            fresh_grains,
            finished_grains,
            envelope: SINE_ENVELOPE,
            finished_grain_indexes: Vec::new(),
        }
    }

    fn selection_start(&self) -> Percentage {
        self.selection_start
    }

    fn set_selection_start(&mut self, start: impl Into<Percentage>) -> &mut Self {
        self.selection_start = start.into();

        if self.selection_start > self.selection_end {
            // move end to "catch up" to the beginning
            self.set_selection_end(self.selection_start);
        }

        self
    }

    fn selection_end(&self) -> Percentage {
        self.selection_end
    }

    fn set_selection_end(&mut self, end: impl Into<Percentage>) -> &mut Self {
        self.selection_end = end.into();

        if self.selection_end < self.selection_start {
            // move beginning to be before the ending
            self.set_selection_start(self.selection_end);
        }

        self
    }

    fn set_grain_len(&mut self, grain_len: impl Into<Duration>) -> &mut Self {
        self.grain_len = self.sanitize_grain_len(grain_len);
        self
    }

    fn set_num_channels(&mut self, num_channels: impl Into<NumChannels>) -> &mut Self {
        let num_channels = num_channels.into();
        self.num_channels = num_channels;

        let num_channels = *num_channels;
        let prev_finished_grains_len = self.finished_grains.len();
        let prev_fresh_grains_len = self.fresh_grains.len();
        let prev_total_num_grains = prev_finished_grains_len + prev_fresh_grains_len;

        // adjust grains to be as long as max number of channels
        if num_channels > prev_total_num_grains {
            let new_grains_to_add = num_channels - prev_total_num_grains;
            self.finished_grains.extend(
                (prev_finished_grains_len..prev_finished_grains_len + new_grains_to_add)
                    .map(|i| Self::new_grain(i as u32)),
            );
        } else if num_channels < prev_total_num_grains {
            // keep as many fresh_grains as possible and delete the rest from finished_grains
            self.fresh_grains.truncate(num_channels);
            let num_grains_left_to_keep = num_channels - self.fresh_grains.len();
            self.finished_grains.truncate(num_grains_left_to_keep);
        }

        self
    }

    fn num_channels(&self) -> NumChannels {
        self.num_channels
    }

    fn set_buffer(&mut self, buffer: Arc<Vec<f32>>) -> &mut Self {
        let buffer_len_samples = buffer.len();
        self.buffer = buffer;

        // find any grains that extend past the current buffer length
        self.finished_grain_indexes.clear();
        self.finished_grain_indexes.extend(
            self.fresh_grains
                .iter()
                .filter(|grain| {
                    grain.end_frame > buffer_len_samples
                        || grain.start_frame >= buffer_len_samples
                        || grain.current_frame >= buffer_len_samples
                })
                .map(|grain| grain.id()),
        );

        info!("{:?}", self.finished_grain_indexes);

        // move grains into the finished list
        self.finished_grain_indexes
            .iter()
            .filter_map(|&i| self.fresh_grains.remove(i))
            .for_each(|mut removed_grain| {
                removed_grain.finished = true;
                self.finished_grains.insert(removed_grain);
            });

        self
    }

    fn refresh_interval(&self) -> u32 {
        self.refresh_interval
    }

    fn set_refresh_interval(&mut self, refresh_interval: u32) -> &mut Self {
        self.refresh_interval = Self::sanitize_refresh_interval(refresh_interval);

        self
    }

    fn next_frame_into_buffer<'a>(
        &mut self,
        frame_data_buffer: &'a mut Vec<f32>,
    ) -> &'a mut Vec<f32> {
        // by only filtering/refreshing grains at an interval, it blends one sound into the other
        // decrease speed of refreshes to blend sounds together
        self.filter_long_grains();
        self.refresh_grains();
        self.increment_refresh_counter();
        self.write_frame_data_into_buffer(frame_data_buffer)
    }

    fn next_frame(&mut self) -> Vec<f32> {
        let mut frame_data_buffer = vec![0.0; self.num_channels().into_inner()];
        self.next_frame_into_buffer(&mut frame_data_buffer);
        frame_data_buffer
    }

    fn set_sample_rate(&mut self, sample_rate: u32) -> &mut Self {
        self.sample_rate = sample_rate;

        self
    }

    fn grain_len(&self) -> Duration {
        self.grain_len
    }
}

// internal logic to support public GranularSynthesizer interface
impl GranularSynthesizer {
    fn sanitize_grain_len(&self, grain_len_min: impl Into<Duration>) -> Duration {
        grain_len_min
            .into()
            // should be <= largest possible length
            .min(Self::GRAIN_LEN_MAX)
            // should be >= smallest possible length
            .max(Self::GRAIN_LEN_MIN)
    }

    fn grain_len_in_samples(&self) -> u32 {
        let samples_per_second = self.sample_rate as f32;
        let grain_len_in_seconds = self.grain_len.as_millis() as f32 / 1000.0;
        (samples_per_second * grain_len_in_seconds) as u32
    }

    /// Iterates through array of grains (1 grain for each channel), and refreshes 1
    /// grain that was previously finished with a new range of buffer indexes.
    fn refresh_grains(&mut self) {
        // get start and end of selection
        let selection_start_index = self.selection_start_in_samples();
        let selection_end_index = self.selection_end_in_samples();
        let grain_len_in_samples = self.grain_len_in_samples();

        // if nothing is selected, there's no use in refreshing grains with empty data
        let selection_is_empty = selection_start_index >= selection_end_index;

        if selection_is_empty {
            return;
        }

        while let Some(finished_grain) = self.finished_grains.pop_first() {
            let smallest_start_index = selection_start_index;
            let largest_start_index = selection_end_index - grain_len_in_samples;

            // get random index inside selection
            let grain_start_index = self
                .rng
                .gen_range(smallest_start_index..=largest_start_index);
            let grain_end_index = grain_start_index + grain_len_in_samples;

            let fresh_grain = Grain::new(
                grain_start_index as usize,
                grain_end_index as usize,
                // keep the same uid as previous grain
                finished_grain.uid,
            );
            self.fresh_grains.insert(fresh_grain);
        }
    }

    fn selection_start_in_samples(&self) -> u32 {
        ((self.buffer.len() as f32 * self.selection_start) as u32)
            .max(0)
            .min(self.buffer.len() as u32)
    }

    fn selection_end_in_samples(&self) -> u32 {
        ((self.buffer.len() as f32 * self.selection_end) as u32)
            .max(0)
            .min(self.buffer.len() as u32)
    }

    fn selection_len_in_samples(&self) -> u32 {
        (self.selection_end_in_samples() - self.selection_start_in_samples()).max(0)
    }

    /// Prevent long grains from lingering when max length and/or selection has changed
    ///
    /// Finds a single grain that exceeds the current selection length and marks it as finished
    /// If no grain is found that exceeds the current selection length, no other action is taken.
    fn filter_long_grains(&mut self) {
        let grain_len_in_samples = self.grain_len_in_samples() as usize;
        let selection_len_in_samples = self.selection_len_in_samples() as usize;

        // find all grains that are too long
        let long_grain_indexes: Vec<_> = self
            .fresh_grains
            .iter()
            .filter(|grain| {
                let remaining_grain_samples = grain.remaining_samples();
                remaining_grain_samples > grain_len_in_samples
                    || remaining_grain_samples > selection_len_in_samples
            })
            .map(|grain| grain.id())
            .collect();

        // move the grains into the finished_grains list
        long_grain_indexes.iter().for_each(|i| {
            self.fresh_grains.remove(*i).map(|mut removed_grain| {
                removed_grain.finished = true;
                self.finished_grains.insert(removed_grain);
            });
        })
    }

    /// Combines current buffer and envelope sample values to calculate a full audio frame
    /// (where each channel gets a single audio output value).
    fn write_frame_data_into_buffer<'a>(
        &mut self,
        frame_data_buffer: &'a mut Vec<f32>,
    ) -> &'a mut Vec<f32> {
        let num_channels_for_frame = self.num_channels().into_inner();
        frame_data_buffer.resize(num_channels_for_frame, 0.0);
        frame_data_buffer
            .iter_mut()
            .zip(self.fresh_grains.iter_mut())
            .for_each(|(channel, grain)| {
                if grain.finished {
                    *channel = 0.0;
                    return;
                }
                let sample_value = self.buffer[grain.current_frame];
                let grain_len = grain.len.max(1) as f32;
                let envelope_percent =
                    ((grain.current_frame - grain.start_frame) as f32) / grain_len;
                let envelope_i = (envelope_percent * self.envelope.len() as f32) as usize;
                let envelope_value = self.envelope[envelope_i];

                *channel = sample_value * envelope_value;

                grain.next_frame();
            });
        frame_data_buffer
    }

    fn increment_refresh_counter(&mut self) {
        self.refresh_counter = self.refresh_counter.wrapping_add(1);
    }
}

#[cfg(test)]
mod test_granular_synthesizer {
    #[cfg(test)]
    mod num_channels {
        use crate::{
            granular_synthesizer::GranularSynthesizer,
            granular_synthesizer_action::GranularSynthesizerAction,
        };
        use std::sync::Arc;

        #[test]
        fn it_should_have_default_num_channels() {
            let synth = GranularSynthesizer::new();

            assert_eq!(
                *synth.num_channels(),
                GranularSynthesizer::DEFAULT_NUM_CHANNELS
            );
            assert_eq!(synth.fresh_grains.len(), 0);
            assert_eq!(synth.finished_grains.len(), 2);
        }

        #[test]
        fn it_should_be_able_to_set_new_num_channels() {
            let mut synth = GranularSynthesizer::new();

            assert_eq!(
                *synth.num_channels(),
                GranularSynthesizer::DEFAULT_NUM_CHANNELS
            );
            assert_eq!(synth.fresh_grains.len(), 0);
            assert_eq!(synth.finished_grains.len(), 2);

            synth.set_num_channels(10);

            assert_eq!(*synth.num_channels(), 10);
            assert_eq!(synth.fresh_grains.len(), 0);
            assert_eq!(synth.finished_grains.len(), 10);
        }

        #[test]
        fn it_should_dynamically_update_grains_when_new_num_channels_set() {
            let mut synth = GranularSynthesizer::new();
            synth.set_buffer(Arc::new(vec![0.0; 1024]));

            assert_eq!(
                *synth.num_channels(),
                GranularSynthesizer::DEFAULT_NUM_CHANNELS
            );
            assert_eq!(synth.fresh_grains.len(), 0);
            assert_eq!(synth.finished_grains.len(), 2);

            // increase channels to get more grains
            synth.set_num_channels(10);

            assert_eq!(*synth.num_channels(), 10);
            assert_eq!(synth.fresh_grains.len(), 0);
            assert_eq!(synth.finished_grains.len(), 10);

            // move some grains to fresh grains list
            synth.next_frame();

            assert_eq!(synth.fresh_grains.len(), 10);
            assert_eq!(synth.finished_grains.len(), 0);

            // reset num channels
            synth.set_num_channels(4);

            // keep as many fresh grains as possible and delete all the remaining finished ones
            assert_eq!(synth.fresh_grains.len(), 4);
            assert_eq!(synth.finished_grains.len(), 0);
        }
    }

    #[cfg(test)]
    mod set_selection {
        use std::sync::Arc;

        use crate::{
            granular_synthesizer::GranularSynthesizer,
            granular_synthesizer_action::GranularSynthesizerAction,
        };

        #[test]
        fn it_should_allow_setting_new_selection() {
            let mut buffer = vec![0.0; 1024];
            buffer.append(&mut vec![1.0; 1024]);
            let mut synth = GranularSynthesizer::new();
            synth.set_buffer(Arc::new(buffer));

            let next_frame = synth.next_frame();
            assert_eq!(next_frame, vec![0.0, 0.0]);

            synth.set_selection_start(0.8).set_selection_end(1.0);

            let next_frame = synth.next_frame();
            assert_eq!(next_frame, vec![1.0, 1.0]);
        }
    }
}
