use crate::grain::Grain;
use crate::granular_synthesizer_action::GranularSynthesizerAction;
use crate::percentage::Percentage;
use crate::{Envelope, EnvelopeType, NumChannels};
use nohash_hasher::IntMap;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;
use std::time::Duration;

/// Accepts a reference to a buffer of Vec<f32> audio sample data.
///
/// Generates random multi-channel audio grain output.
#[derive(Debug, Clone)]
pub struct GranularSynthesizer {
    /// How many channels of sound to generate per frame
    num_channels: NumChannels,

    /// Sample rate of the surrounding context
    sample_rate: u32,

    /// External audio buffer that this GranularSynthesizer should read grains from
    buffer: Arc<Vec<f32>>,

    /// used to generate random indexes (should be decently fast)
    rng: SmallRng,

    /// This is the max length of audio that each grain can play--
    /// if the selected portion of the buffer is smaller than the
    /// grain size, then the final grain size will be smaller to
    /// stay within bounds of the selected audio
    grain_len: Duration,

    /// This is how long the granular synthesizer should wait before initializing
    /// grains for the first time. Since all grains get (nearly) the same length,
    /// if there is no delay in grain initialization, then they will all start/stop
    /// at the same time.
    grain_initialization_delay: Duration,

    /// The minimum index that samples can be taken from,
    /// ranging from 0.0 -> 1.0 (i.e. Duration of the buffer)
    selection_start: Percentage,

    /// The maximum index that samples can be taken from,
    /// ranging from 0.0 -> 1.0 (i.e. percentage of the buffer)
    selection_end: Percentage,

    /// This is a counter that gets incremented on every frame.
    /// This allows performing some actions (such as replacing grains) to occur
    /// only so often.
    frame_count: u32,

    fresh_grains: IntMap<u32, Grain>,

    finished_grains: IntMap<u32, Grain>,

    uninitialized_grains: IntMap<u32, Grain>,

    /// Volume envelope used for controlling the volume of each grain's playback
    envelope: Envelope,

    /// cached for more efficient processing
    selection_end_in_samples: u32,

    /// cached for more efficient processing
    selection_start_in_samples: u32,
}

impl GranularSynthesizerAction for GranularSynthesizer {
    fn new() -> Self {
        let default_buffer = Arc::new(Vec::new());
        let mut uninitialized_grains = IntMap::default();

        for grain in (0..Self::DEFAULT_NUM_CHANNELS).map(|i| Self::new_grain(i as u32)) {
            uninitialized_grains.insert(grain.uid, grain);
        }

        let fresh_grains = IntMap::default();
        let finished_grains = IntMap::default();

        Self {
            sample_rate: Self::DEFAULT_SAMPLE_RATE,
            buffer: default_buffer,
            rng: SmallRng::from_entropy(),
            grain_len: Self::DEFAULT_GRAIN_LEN,
            selection_start: Percentage::from(0.0),
            selection_end: Percentage::from(1.0),
            num_channels: NumChannels::new(Self::DEFAULT_NUM_CHANNELS),
            frame_count: 0,
            uninitialized_grains,
            fresh_grains,
            finished_grains,
            envelope: Envelope::new_sine(),
            grain_initialization_delay: Self::DEFAULT_GRAIN_INITIALIZATION_DELAY,
            selection_end_in_samples: 0,
            selection_start_in_samples: 0,
        }
    }

    fn selection_start(&self) -> Percentage {
        self.selection_start
    }

    fn selection_end(&self) -> Percentage {
        self.selection_end
    }

    fn set_buffer(&mut self, buffer: Arc<Vec<f32>>) -> &mut Self {
        self.buffer = buffer;
        self.selection_end_in_samples = self.calculate_selection_end_in_samples();
        self.selection_start_in_samples = self.calculate_selection_start_in_samples();
        self.fresh_grains.drain().for_each(|(uid, mut grain)| {
            grain.init = false;
            self.uninitialized_grains.insert(uid, grain);
        });

        self
    }

    fn set_selection_start(&mut self, start: impl Into<Percentage>) -> &mut Self {
        self.selection_start = start.into();

        if self.selection_start > self.selection_end {
            // move end to "catch up" to the beginning
            self.set_selection_end(self.selection_start);
        }

        self.selection_start_in_samples = self.calculate_selection_start_in_samples();

        self
    }

    fn set_selection_end(&mut self, end: impl Into<Percentage>) -> &mut Self {
        self.selection_end = end.into();

        if self.selection_end < self.selection_start {
            // move beginning to be before the ending
            self.set_selection_start(self.selection_end);
        }

        self.selection_end_in_samples = self.calculate_selection_end_in_samples();

        self
    }

    fn set_grain_len(&mut self, grain_len: impl Into<Duration>) -> &mut Self {
        self.grain_len = self.sanitize_grain_len(grain_len);
        self
    }

    fn set_num_channels(&mut self, num_channels: impl Into<NumChannels>) -> &mut Self {
        self.num_channels = num_channels.into();
        self
    }

    fn grain_initialization_delay(&self) -> Duration {
        self.grain_initialization_delay
    }

    fn set_grain_initialization_delay(&mut self, delay: impl Into<Duration>) -> &mut Self {
        self.grain_initialization_delay = Self::sanitize_grain_initialization_delay(delay.into());
        self
    }

    fn num_channels(&self) -> NumChannels {
        self.num_channels
    }

    fn next_frame_into_buffer<'a>(
        &mut self,
        frame_data_buffer: &'a mut Vec<f32>,
    ) -> &'a mut Vec<f32> {
        self.synchronize_num_grains_with_channels();
        self.initialize_an_uninitialized_grain();
        self.filter_fresh_grains();
        self.refresh_finished_grains();
        self.increment_frame_count();
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

    fn set_envelope(&mut self, envelope_type: EnvelopeType) -> &mut Self {
        self.envelope = envelope_type.into();
        self
    }
}

// internal logic to support public GranularSynthesizer interface
impl GranularSynthesizer {
    /// Make sure num_channels is equal to the total number of grains
    fn synchronize_num_grains_with_channels(&mut self) -> &mut Self {
        let num_channels = *self.num_channels();
        let total_num_grains = self.total_num_grains();

        if num_channels > total_num_grains {
            let new_grains_to_add = num_channels - total_num_grains;
            for grain in ((total_num_grains + 1)..(total_num_grains + 1 + new_grains_to_add))
                .map(|i| Self::new_grain(i as u32))
            {
                self.uninitialized_grains.insert(grain.uid, grain);
            }
        } else if num_channels < total_num_grains {
            // keep as many fresh_grains as possible, and delete the rest from finished_grains

            if num_channels < self.fresh_grains.len() {
                for i in num_channels..self.fresh_grains.len() {
                    self.fresh_grains.remove(&(i as u32));
                }
            }

            // keep as many finished_grains as possible, and delete the rest from uninitialized_grains
            let num_finished_grains_left_to_keep = num_channels - self.fresh_grains.len();
            if num_finished_grains_left_to_keep < self.finished_grains.len() {
                for i in num_finished_grains_left_to_keep..self.fresh_grains.len() {
                    self.finished_grains.remove(&(i as u32));
                }
            }

            // delete the rest from uninitialized_grains
            let num_uninitialized_grains_left_to_keep =
                num_channels - self.fresh_grains.len() - self.finished_grains.len();

            if num_uninitialized_grains_left_to_keep < self.uninitialized_grains.len() {
                for i in num_uninitialized_grains_left_to_keep..self.uninitialized_grains.len() {
                    self.uninitialized_grains.remove(&(i as u32));
                }
            }
        }

        self
    }

    fn total_num_grains(&self) -> usize {
        self.uninitialized_grains.len() + self.finished_grains.len() + self.fresh_grains.len()
    }

    ///  helps make sure we are only initializing grains every `n` samples
    fn frame_aligns_with_delay_interval(&self) -> bool {
        let grain_initialization_delay_in_samples = self.grain_initialization_delay_in_samples();
        grain_initialization_delay_in_samples == 0
            || self.frame_count % grain_initialization_delay_in_samples == 0
    }

    /// Move one uninitialized grain into the fresh_grains set
    /// (do this once every `grain_initialization_delay` frames)
    fn initialize_an_uninitialized_grain(&mut self) -> &mut Self {
        if self.buffer_selection_is_empty() {
            return self;
        }

        // make sure we are only initializing grains every `n` samples
        if !self.frame_aligns_with_delay_interval() {
            return self;
        }

        // uninitialized grain should be moved into the fresh_grains list--
        // the new, refreshed grain should use the same uid as the uninitialized one
        let Some((_, Grain {  uid, .. })) = self.uninitialized_grains.iter().next() else {
            return self;
        };

        let uid = *uid;
        self.uninitialized_grains.remove(&uid);

        const INIT: bool = true;
        let (grain_start_index, grain_end_index) = self.get_grain_random_start_and_end();
        let fresh_grain = Grain::new(grain_start_index, grain_end_index, uid, INIT);
        self.fresh_grains.insert(fresh_grain.uid, fresh_grain);

        self
    }

    fn grain_initialization_delay_in_samples(&self) -> u32 {
        let samples_per_second = self.sample_rate as f64;
        (samples_per_second * self.grain_initialization_delay.as_secs_f64()) as u32
    }

    fn sanitize_grain_len(&self, grain_len_min: impl Into<Duration>) -> Duration {
        grain_len_min
            .into()
            // should be <= largest possible length
            .min(Self::GRAIN_LEN_MAX)
            // should be >= smallest possible length
            .max(Self::GRAIN_LEN_MIN)
    }

    fn grain_len_in_samples(&mut self) -> u32 {
        let samples_per_second = self.sample_rate as f32;
        let grain_len_in_seconds = self.grain_len.as_secs_f32() as f32;
        (samples_per_second * grain_len_in_seconds) as u32
    }

    /// Iterates through array of grains (1 grain for each channel), and refreshes 1
    /// grain that was previously finished with a new range of buffer indexes.
    fn refresh_finished_grains(&mut self) {
        if self.buffer_selection_is_empty() {
            return;
        }

        let uids: Vec<_> = self
            .finished_grains
            .drain()
            .map(|(_, grain)| grain.uid)
            .collect();

        for uid in uids {
            let (grain_start_index, grain_end_index) = self.get_grain_random_start_and_end();
            let fresh_grain = Grain::new(
                grain_start_index,
                grain_end_index,
                // keep the same uid as previous grain
                uid,
                true,
            );
            self.fresh_grains.insert(uid, fresh_grain);
        }
    }

    fn buffer_selection_is_empty(&self) -> bool {
        let selection_start_index = self.selection_start_in_samples();
        let selection_end_index = self.selection_end_in_samples();

        // if nothing is selected, there's no use in refreshing grains with empty data
        selection_start_index >= selection_end_index
    }

    fn get_grain_random_start_and_end(&mut self) -> (usize, usize) {
        // get start and end of selection
        let selection_start_index = self.selection_start_in_samples();
        let selection_end_index = self.selection_end_in_samples();
        let grain_len_in_samples = self.grain_len_in_samples();

        if self.buffer_selection_is_empty() {
            return (selection_start_index as usize, selection_end_index as usize);
        }

        let smallest_start_index = selection_start_index;
        let range_would_be_empty = (selection_end_index < grain_len_in_samples)
            || ((selection_end_index - grain_len_in_samples) < smallest_start_index);

        let largest_start_index = if range_would_be_empty {
            smallest_start_index
        } else {
            selection_end_index - grain_len_in_samples
        };

        // get random starting index inside selection
        let grain_start_index = if smallest_start_index < largest_start_index {
            self.rng
                .gen_range(smallest_start_index..=largest_start_index)
        } else {
            smallest_start_index
        };

        // all grains have the same length (for now)
        let grain_end_index = grain_start_index + grain_len_in_samples;

        (grain_start_index as usize, grain_end_index as usize)
    }

    fn selection_start_in_samples(&self) -> u32 {
        self.selection_start_in_samples
    }

    /// If using on a hot code path, prefer the cached `selection_start_in_samples()` value instead
    fn calculate_selection_start_in_samples(&self) -> u32 {
        ((self.buffer.len() as f32 * self.selection_start) as u32)
            .max(0)
            .min(self.buffer.len() as u32)
    }

    fn selection_end_in_samples(&self) -> u32 {
        self.selection_end_in_samples
    }

    /// If using on a hot code path, prefer the cached `selection_end_in_samples()` value instead
    fn calculate_selection_end_in_samples(&self) -> u32 {
        ((self.buffer.len() as f32 * self.selection_end) as u32)
            .max(0)
            .min(self.buffer.len() as u32)
    }

    /// Moves any grains that exceed the selected portion of the buffer or are
    /// finished into the finished_grains set.
    fn filter_fresh_grains(&mut self) {
        if !self.frame_aligns_with_delay_interval() {
            return;
        }

        // assuming that the end_frame of a Grain is guaranteed to be
        // larger than the start_ and/or current_frame of a Grain,
        // then it's safe to check only one side of the selection
        let filter_long_grains = |grain: &&Grain| -> bool {
            (grain.current_frame as u32) < self.selection_start_in_samples()
                || (grain.end_frame as u32) > self.selection_end_in_samples()
        };

        let long_grain_uid = self
            .fresh_grains
            .values()
            .find(|grain| filter_long_grains(grain))
            .map(|grain| grain.uid);

        // move long grains into the uninitialized_grains list--
        // this list gets refreshed less frequently than the finished_grains list,
        // which helps prevent all the grains from aligning simultaneously
        // when changing buffer selection (this can cause massive clipping)
        if let Some(long_grain_uid) = long_grain_uid {
            self.fresh_grains
                .remove(&long_grain_uid)
                .map(|mut removed_grain| {
                    removed_grain.finished = true;
                    removed_grain.init = false;
                    self.uninitialized_grains
                        .insert(long_grain_uid, removed_grain);
                });
        }

        fn filter_finished_grains(grain: &&Grain) -> bool {
            grain.finished
        }

        let finished_grain_uid = self
            .fresh_grains
            .values()
            .find(|grain| filter_finished_grains(grain))
            .map(|grain| grain.uid);

        // move finished grains into the finished_grains list--
        // this list gets refreshed more frequently than the
        // uninitialized grains list
        if let Some(finished_grain_uid) = finished_grain_uid {
            self.fresh_grains
                .remove(&finished_grain_uid)
                .map(|mut removed_grain| {
                    removed_grain.finished = true;
                    self.finished_grains
                        .insert(finished_grain_uid, removed_grain);
                });
        }
    }

    /// Combines current buffer and envelope sample values to calculate a full audio frame
    /// (where each channel gets a single audio output value).
    fn write_frame_data_into_buffer<'a>(
        &mut self,
        frame_data_buffer: &'a mut Vec<f32>,
    ) -> &'a mut Vec<f32> {
        let num_channels_for_frame = self.num_channels().into_inner();
        frame_data_buffer.resize(num_channels_for_frame, 0.0);
        frame_data_buffer.fill(0.0);
        frame_data_buffer
            .iter_mut()
            .zip(self.fresh_grains.values_mut())
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

    fn increment_frame_count(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
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
        use std::{sync::Arc, time::Duration};

        #[test]
        fn it_should_have_default_num_channels() {
            let synth = GranularSynthesizer::new();

            assert_eq!(
                *synth.num_channels(),
                GranularSynthesizer::DEFAULT_NUM_CHANNELS
            );
            assert_eq!(synth.uninitialized_grains.len(), 2);
            assert_eq!(synth.fresh_grains.len(), 0);
            assert_eq!(synth.finished_grains.len(), 0);
        }

        #[test]
        fn it_should_be_able_to_set_new_num_channels() {
            let mut synth = GranularSynthesizer::new();

            assert_eq!(
                *synth.num_channels(),
                GranularSynthesizer::DEFAULT_NUM_CHANNELS
            );
            assert_eq!(*synth.num_channels(), 2);
            assert_eq!(synth.uninitialized_grains.len(), 2);
            assert_eq!(synth.fresh_grains.len(), 0);
            assert_eq!(synth.finished_grains.len(), 0);

            synth.set_num_channels(10);
            synth.next_frame();

            assert_eq!(*synth.num_channels(), 10);
            assert_eq!(synth.uninitialized_grains.len(), 10);
            assert_eq!(synth.fresh_grains.len(), 0);
            assert_eq!(synth.finished_grains.len(), 0);
        }

        #[test]
        fn it_should_dynamically_update_grains_when_new_num_channels_set() {
            let mut synth = GranularSynthesizer::new();
            synth
                .set_buffer(Arc::new(vec![0.0; 1024]))
                .set_grain_initialization_delay(Duration::MAX);

            assert_eq!(
                *synth.num_channels(),
                GranularSynthesizer::DEFAULT_NUM_CHANNELS
            );
            assert_eq!(*synth.num_channels(), 2);
            assert_eq!(synth.uninitialized_grains.len(), 2);
            assert_eq!(synth.fresh_grains.len(), 0);
            assert_eq!(synth.finished_grains.len(), 0);

            // increase channels to get more grains
            synth.set_num_channels(10);
            synth.next_frame();

            assert_eq!(*synth.num_channels(), 10);
            assert_eq!(synth.uninitialized_grains.len(), 9);
            assert_eq!(synth.fresh_grains.len(), 1);
            assert_eq!(synth.finished_grains.len(), 0);

            // reset num channels
            synth.set_num_channels(4);
            synth.next_frame();

            // keep as many fresh grains as possible and delete all the remaining finished ones
            assert_eq!(*synth.num_channels(), 4);
            assert_eq!(synth.uninitialized_grains.len(), 3);
            assert_eq!(synth.fresh_grains.len(), 1);
            assert_eq!(synth.finished_grains.len(), 0);
        }

        #[test]
        fn it_should_allow_setting_new_num_channels_to_large_amount() {
            let mut synth = GranularSynthesizer::new();
            synth
                .set_buffer(Arc::new(vec![1.0; 1024]))
                .set_envelope(crate::EnvelopeType::All1)
                .set_grain_initialization_delay(Duration::ZERO);

            for _ in 0..=2 {
                synth.next_frame();
            }

            let next_frame = synth.next_frame();

            assert_eq!(next_frame.len(), 2);
            assert_eq!(next_frame, vec![1.0; 2]);

            synth.set_num_channels(100);

            for _ in 0..=200 {
                synth.next_frame();
            }

            let next_frame = synth.next_frame();

            assert_eq!(next_frame.len(), 100);
            assert_eq!(next_frame, vec![1.0; 100]);
        }

        #[test]
        fn it_should_allow_large_numbers_of_channels() {
            let mut synth = GranularSynthesizer::new();
            synth
                .set_buffer(Arc::new(vec![1.0; 1024]))
                .set_envelope(crate::EnvelopeType::All1)
                .set_grain_initialization_delay(Duration::ZERO)
                .set_num_channels(200);

            for _ in 0..=200 {
                synth.next_frame();
            }

            let next_frame = synth.next_frame();

            assert_eq!(next_frame.len(), 200);
            assert_eq!(next_frame, vec![1.0; 200]);
        }
    }

    #[cfg(test)]
    mod set_selection {
        use std::{sync::Arc, time::Duration};

        use crate::{
            granular_synthesizer::GranularSynthesizer,
            granular_synthesizer_action::GranularSynthesizerAction, EnvelopeType,
        };

        #[test]
        fn it_should_allow_setting_new_selection() {
            let mut buffer = vec![0.0; 1024];
            buffer.append(&mut vec![1.0; 1024]);
            let mut synth = GranularSynthesizer::new();
            synth
                .set_buffer(Arc::new(buffer))
                .set_envelope(EnvelopeType::All1)
                .set_selection_start(0.0)
                .set_selection_end(0.4)
                .set_grain_initialization_delay(Duration::ZERO);

            let next_frame = synth.next_frame();
            assert_eq!(next_frame, vec![0.0, 0.0]);

            synth.set_selection_start(0.6).set_selection_end(1.0);

            // allow both channels to get initialized
            synth.next_frame();
            let next_frame = synth.next_frame();

            assert_eq!(next_frame, vec![1.0, 1.0]);
        }

        #[test]
        fn it_should_allow_setting_many_new_selection_while_producing_grains() {
            // prepare buffer with "colored" data
            let mut buffer = vec![0.0; 1024];
            buffer.append(&mut vec![0.25; 1024]);
            buffer.append(&mut vec![0.5; 1024]);
            buffer.append(&mut vec![0.75; 1024]);

            let mut synth = GranularSynthesizer::new();
            synth
                .set_buffer(Arc::new(buffer))
                .set_envelope(EnvelopeType::All1)
                .set_grain_initialization_delay(Duration::ZERO)
                .set_num_channels(100);

            for _ in 0..200 {
                synth.next_frame();
            }

            // resize buffer selection while generating frames
            for selection_end in 0..100 {
                synth.set_selection_start(0.25);
                let selection_end_progress = selection_end as f32 / 100.0;
                synth.set_selection_end(0.25 + 0.25 * selection_end_progress);
                synth.next_frame();
            }

            for _ in 0..200 {
                synth.next_frame();
            }

            let next_frame = synth.next_frame();
            assert_eq!(next_frame, vec![0.25; 100]);

            // resize buffer selection while generating frames
            for selection_end in 0..1000 {
                synth.set_selection_start(0.5);
                let selection_end_progress = selection_end as f32 / 1000.0;
                synth.set_selection_end(0.5 + 0.25 * selection_end_progress);
                synth.next_frame();
            }

            for _ in 0..200 {
                synth.next_frame();
            }

            let next_frame = synth.next_frame();
            assert_eq!(next_frame, vec![0.5; 100]);
        }

        #[test]
        fn it_should_allow_setting_zero_length_selection() {
            // prepare buffer with "colored" data
            let mut buffer = vec![0.0; 1024];
            buffer.append(&mut vec![0.25; 1024]);
            buffer.append(&mut vec![0.5; 1024]);
            buffer.append(&mut vec![0.75; 1024]);

            let mut synth = GranularSynthesizer::new();
            synth
                .set_buffer(Arc::new(buffer))
                .set_envelope(EnvelopeType::All1)
                .set_grain_initialization_delay(Duration::ZERO)
                .set_num_channels(100);

            // generate frames with full buffer selected
            for _ in 0..1000 {
                synth.next_frame();
            }

            // generate frames with zero-length buffer selected
            synth.set_selection_start(0.25).set_selection_end(0.25);

            for _ in 0..1000 {
                synth.next_frame();
            }

            let next_frame = synth.next_frame();
            assert_eq!(next_frame, vec![0.0; 100]);

            // generate frames with normal buffer selected
            synth.set_selection_start(0.75).set_selection_end(1.0);

            for _ in 0..1000 {
                synth.next_frame();
            }

            let next_frame = synth.next_frame();
            assert_eq!(next_frame, vec![0.75; 100]);
        }

        #[test]
        fn it_should_allow_setting_many_new_selection_with_many_channels() {
            // prepare buffer with "colored" data
            let mut buffer = vec![0.0; 1024];
            buffer.append(&mut vec![0.25; 1024]);
            buffer.append(&mut vec![0.5; 1024]);
            buffer.append(&mut vec![0.75; 1024]);

            let mut synth = GranularSynthesizer::new();
            synth
                .set_buffer(Arc::new(buffer))
                .set_envelope(EnvelopeType::All1)
                .set_grain_initialization_delay(Duration::ZERO)
                .set_num_channels(250);

            for _ in 0..200 {
                synth.next_frame();
            }

            // resize buffer selection while generating frames
            for selection_end in 0..10 {
                synth.set_selection_start(0.25);
                let selection_end_progress = selection_end as f32 / 10.0;
                synth.set_selection_end(0.25 + 0.25 * selection_end_progress);

                for _ in 0..100 {
                    synth.next_frame();
                }
            }

            let next_frame = synth.next_frame();
            assert_eq!(next_frame, vec![0.25; 250]);
        }
    }

    #[cfg(test)]
    mod playback {
        use std::{sync::Arc, time::Duration};

        use crate::{
            granular_synthesizer::GranularSynthesizer,
            granular_synthesizer_action::GranularSynthesizerAction,
        };

        #[test]
        fn it_should_return_sequential_frames_of_audio_data() {
            let mut synth = GranularSynthesizer::new();
            let buffer: Vec<_> = (0..5000).map(|i| i as f32).collect();
            synth
                .set_buffer(Arc::new(buffer))
                .set_envelope(crate::EnvelopeType::All1)
                .set_grain_initialization_delay(Duration::ZERO);

            // grain 1 initialized
            synth.next_frame();
            // grain 2 initialized
            let frame_1 = synth.next_frame();
            // grain 1 and 2 now both have had 2 frames of data
            let frame_2 = synth.next_frame();

            assert_eq!(frame_1[0] + 1.0, frame_2[0]);
            assert_eq!(frame_1[1] + 1.0, frame_2[1]);
        }
    }

    #[cfg(test)]
    mod grain_initialization_delay {
        use std::{sync::Arc, time::Duration};

        use crate::{
            granular_synthesizer::GranularSynthesizer,
            granular_synthesizer_action::GranularSynthesizerAction,
        };

        #[test]
        fn it_should_allow_medium_sized_delays() {
            let mut synth = GranularSynthesizer::new();
            synth
                .set_buffer(Arc::new(vec![0.0; 1024]))
                .set_envelope(crate::EnvelopeType::All1)
                .set_grain_initialization_delay(Duration::from_millis(123));

            // grain 1 initialized
            synth.next_frame();
        }
    }
}
