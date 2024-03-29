use crate::GranularSynthesizerAction;
use crate::GranularSynthesizerGrain as Grain;
use crate::LazyCached;
use crate::Percentage;
use crate::{Envelope, EnvelopeType, NumChannels};
use nohash_hasher::IntMap;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::cmp::Ordering;
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
    selection_end_in_samples: LazyCached<u32>,

    /// cached for more efficient processing
    selection_start_in_samples: LazyCached<u32>,
}

impl GranularSynthesizerAction for GranularSynthesizer {
    fn new() -> Self {
        let seed: <SmallRng as SeedableRng>::Seed = Default::default();
        GranularSynthesizer::from_seed(seed)
    }

    fn selection_start(&self) -> Percentage {
        self.selection_start
    }

    fn selection_end(&self) -> Percentage {
        self.selection_end
    }

    fn set_buffer(&mut self, buffer: Arc<Vec<f32>>) -> &mut Self {
        self.buffer = buffer;

        // we know that none of the fresh_grains are valid anymore, since it's a new buffer
        self.fresh_grains.drain().for_each(|(uid, mut grain)| {
            grain.is_init = false;
            self.uninitialized_grains.insert(uid, grain);
        });

        self.selection_start_in_samples.invalidate();
        self.selection_end_in_samples.invalidate();

        self
    }

    fn set_selection_start(&mut self, start: impl Into<Percentage>) -> &mut Self {
        self.selection_start = start.into();

        if self.selection_start > self.selection_end {
            // move end to "catch up" to the beginning
            self.set_selection_end(self.selection_start);
            self.selection_end_in_samples.invalidate();
        }

        self.selection_start_in_samples.invalidate();

        self
    }

    fn set_selection_end(&mut self, end: impl Into<Percentage>) -> &mut Self {
        self.selection_end = end.into();

        if self.selection_end < self.selection_start {
            // move beginning to be before the ending
            self.set_selection_start(self.selection_end);
            self.selection_start_in_samples.invalidate();
        }

        self.selection_end_in_samples.invalidate();

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

    fn next_frame_into_buffer<'a>(&mut self, frame_data_buffer: &'a mut [f32]) -> &'a mut [f32] {
        self.run_next_frame_pipeline(frame_data_buffer, false)
    }

    fn next_frame(&mut self) -> Vec<f32> {
        let mut frame_data_buffer = vec![0.0; self.num_channels().into_inner()];
        self.run_next_frame_pipeline(&mut frame_data_buffer, true);
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
    /// Allows seeding random number generator manually for consistent snapshot testing
    pub fn from_seed(seed: <SmallRng as SeedableRng>::Seed) -> Self {
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
            rng: SmallRng::from_seed(seed),
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
            selection_end_in_samples: LazyCached::new_uncached(),
            selection_start_in_samples: LazyCached::new_uncached(),
        }
    }

    /// This is the pipeline for generating a frame of audio--it can be shared
    /// between the pipeline that allocates a new Vec and the one that uses
    /// an existing reference to a buffer to write data
    fn run_next_frame_pipeline<'a>(
        &mut self,
        frame_data_buffer: &'a mut [f32],
        is_new_buffer: bool,
    ) -> &'a mut [f32] {
        self.synchronize_num_grains_with_channels();
        self.initialize_an_uninitialized_grain();
        self.refresh_finished_grains();
        self.increment_frame_count();
        self.write_frame_data_into_buffer(frame_data_buffer, is_new_buffer)
    }

    /// Make sure num_channels is equal to the total number of grains
    fn synchronize_num_grains_with_channels(&mut self) -> &mut Self {
        let num_channels = *self.num_channels();
        let total_num_grains = self.total_num_grains();

        match num_channels.cmp(&total_num_grains) {
            Ordering::Greater => {
                let new_grains_to_add = num_channels - total_num_grains;
                for grain in (total_num_grains..(total_num_grains + new_grains_to_add))
                    .map(|i| Self::new_grain(i as u32))
                {
                    self.uninitialized_grains.insert(grain.uid, grain);
                }
            }
            Ordering::Less => {
                // get rid of all grains that exceed the total number of channels
                let num_channels = *self.num_channels as u32;
                let filter_grain = |_: &u32, grain: &mut Grain| grain.uid < num_channels;

                self.fresh_grains.retain(filter_grain);
                self.finished_grains.retain(filter_grain);
                self.uninitialized_grains.retain(filter_grain)
            }
            Ordering::Equal => {}
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

        // this allows grains to emanate from the "center" of the listener's
        // aural awareness, given that the downmixing function is taking
        // panning into account when downmixing all the channels down to 2
        let grains: Vec<_> = self.uninitialized_grains.values().collect();

        // note: this is technically necessary to ensure grains are properly
        // sorted, since HashMap's iteration order is unspecified, but I've found
        // (at least for now) that it does in the correct order. Since this
        // decreases runtime performance by ~25%, I'm leaving it out for now
        // grains.sort_by_key(|grain| grain.uid);

        let half_way = self.uninitialized_grains.len().min(*self.num_channels) / 2;

        // uninitialized grain should be moved into the fresh_grains list--
        // the new, refreshed grain should use the same uid as the uninitialized one
        let Some(Grain {  uid, .. }) = grains.get(half_way) else {
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
        let selection_start_index = self.selection_start_in_samples();
        let selection_end_index = self.selection_end_in_samples();
        let selection_len_in_samples = selection_end_index - selection_start_index;

        let samples_per_second = self.sample_rate as f32;
        let grain_len_in_seconds = self.grain_len.as_secs_f32();
        let grain_len_in_samples = (samples_per_second * grain_len_in_seconds) as u32;

        selection_len_in_samples.min(grain_len_in_samples)
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

    fn buffer_selection_is_empty(&mut self) -> bool {
        let selection_start_index = self.selection_start_in_samples();
        let selection_end_index = self.selection_end_in_samples();

        // if nothing is selected, there's no use in refreshing grains with empty data
        selection_start_index >= selection_end_index
    }

    fn get_grain_random_start_and_end(&mut self) -> (usize, usize) {
        // get start and end of selection
        let selection_start_index = self.selection_start_in_samples();
        let selection_end_index = self.selection_end_in_samples();

        if self.buffer_selection_is_empty() {
            return (selection_start_index as usize, selection_end_index as usize);
        }

        let grain_len_in_samples = self.grain_len_in_samples();

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

    fn selection_start_in_samples(&mut self) -> u32 {
        fn calculate_selection_start_in_samples(
            buffer_len: usize,
            selection_start: Percentage,
        ) -> u32 {
            ((buffer_len as f32 * selection_start) as u32)
                .max(0)
                .min(buffer_len as u32)
        }

        let buffer_len = self.buffer.len();
        let selection_start = self.selection_start;
        *self
            .selection_start_in_samples
            .get(|| calculate_selection_start_in_samples(buffer_len, selection_start))
    }

    fn selection_end_in_samples(&mut self) -> u32 {
        fn calculate_selection_end_in_samples(buffer_len: usize, selection_end: Percentage) -> u32 {
            ((buffer_len as f32 * selection_end) as u32)
                .max(0)
                .min(buffer_len as u32)
        }

        let buffer_len = self.buffer.len();
        let selection_end = self.selection_end;
        *self
            .selection_end_in_samples
            .get(|| calculate_selection_end_in_samples(buffer_len, selection_end))
    }

    /// Combines current buffer and envelope sample values to calculate a full audio frame
    /// (where each channel gets a single audio output value).
    fn write_frame_data_into_buffer<'a>(
        &mut self,
        frame_data_buffer: &'a mut [f32],
        is_new_buffer: bool,
    ) -> &'a mut [f32] {
        let selection_start_in_samples = self.selection_start_in_samples();
        let selection_end_in_samples = self.selection_end_in_samples();
        let mut finished_grain_uid = None;

        // if writing to a previously used buffer, make sure that buffer
        // has been cleaned up first, so that previous output does
        // not linger into the next frame
        if !is_new_buffer {
            frame_data_buffer.fill(0.0);
        }

        // spread out the grains into a vec with the same number of slots as there are channels
        let mut grains_as_channels = vec![None; *self.num_channels + 1];
        for grain in self.fresh_grains.values_mut() {
            let uid = grain.uid as usize;

            // if the uid exceeds the number of channels, we don't need its output
            if uid >= grains_as_channels.len() {
                continue;
            }

            // safe to store/deref these pointers since they are temporary and unique pointers
            grains_as_channels[uid] = Some(grain as *mut Grain);
        }

        frame_data_buffer
            .iter_mut()
            .zip(grains_as_channels.into_iter())
            .for_each(|(channel, grain)| {
                let Some(grain) = grain else {
                    *channel = 0.0;
                    return;
                };

                let grain = unsafe { &mut *grain };

                if grain.calculate_exceeds_buffer_selection(
                    selection_start_in_samples,
                    selection_end_in_samples,
                ) {
                    // mark for moving into appropriate hash map later
                    grain.exceeds_buffer_selection = true;
                }

                if grain.is_finished {
                    // mark for moving into the finished or uninitialized hash map later
                    if finished_grain_uid.is_none() {
                        finished_grain_uid.replace(grain.uid);
                    }

                    // output for this grain should be 0
                    *channel = 0.0;
                    return;
                }

                // get final sample value for the current grain/channel
                let sample_value = self.buffer[grain.current_frame];
                let grain_len = grain.len.max(1) as f32;
                let envelope_percent =
                    ((grain.current_frame - grain.start_frame) as f32) / grain_len;
                let envelope_i = (envelope_percent * self.envelope.len() as f32) as usize;
                let envelope_value = self.envelope[envelope_i];

                *channel = sample_value * envelope_value;

                grain.next_frame();
            });

        // move a finished grain into the finished_grains list
        // this list gets refreshed more frequently than the
        // uninitialized grains list
        if let Some(finished_grain_uid) = finished_grain_uid {
            if let Some(mut removed_grain) = self.fresh_grains.remove(&finished_grain_uid) {
                removed_grain.is_finished = true;
                if removed_grain.exceeds_buffer_selection {
                    self.uninitialized_grains
                        .insert(finished_grain_uid, removed_grain);
                } else {
                    self.finished_grains
                        .insert(finished_grain_uid, removed_grain);
                }
            }
        }

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
            granular_synthesizer::GranularSynthesizerAction,
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
            assert_eq!(synth.uninitialized_grains.len(), 4);
            assert_eq!(synth.fresh_grains.len(), 0);
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

            for _ in 0..=100 {
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
                .set_num_channels(250);

            for _ in 0..(GranularSynthesizer::DEFAULT_SAMPLE_RATE * 10) {
                synth.next_frame();
            }

            for _ in 0..44 {
                synth.next_frame();
            }

            let next_frame = synth.next_frame();

            assert_eq!(next_frame.len(), 250);
            assert_eq!(next_frame, vec![1.0; 250]);
        }
    }

    #[cfg(test)]
    mod set_selection {
        use std::{sync::Arc, time::Duration};

        use crate::{
            granular_synthesizer::GranularSynthesizer,
            granular_synthesizer::GranularSynthesizerAction, EnvelopeType,
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

            // allow grain playing from previous selection to die
            for _ in 0..synth.sample_rate {
                synth.next_frame();
            }

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

            for _ in 0..500 {
                synth.next_frame();
            }

            // resize buffer selection while generating frames
            for selection_end in 0..100 {
                synth.set_selection_start(0.25);
                let selection_end_progress = selection_end as f32 / 100.0;
                synth.set_selection_end(0.25 + 0.25 * selection_end_progress);
                synth.next_frame();
            }

            for _ in 0..500 {
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

            for _ in 0..500 {
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

            for _ in 0..500 {
                synth.next_frame();
            }

            // resize buffer selection while generating frames
            for selection_end in 0..10 {
                synth.set_selection_start(0.25);
                let selection_end_progress = selection_end as f32 / 10.0;
                synth.set_selection_end(0.25 + 0.25 * selection_end_progress);

                for _ in 0..500 {
                    synth.next_frame();
                }
            }

            let next_frame = synth.next_frame();
            assert_eq!(next_frame, vec![0.25; 250]);
        }

        #[test]
        fn should_allow_going_from_high_to_low_num_channels() {
            let mut synth = GranularSynthesizer::new();
            synth
                .set_buffer(Arc::new(vec![1.0; 1024]))
                .set_envelope(crate::EnvelopeType::All1)
                .set_grain_initialization_delay(Duration::ZERO)
                .set_num_channels(250);

            for _ in 0..500 {
                synth.next_frame();
            }

            assert_eq!(synth.next_frame(), vec![1.0; 250]);

            synth.set_num_channels(50);

            for _ in 0..500 {
                synth.next_frame();
            }

            assert_eq!(synth.next_frame(), vec![1.0; 50]);
        }
    }

    #[cfg(test)]
    mod playback {
        use std::{sync::Arc, time::Duration};

        use crate::{
            granular_synthesizer::GranularSynthesizer,
            granular_synthesizer::GranularSynthesizerAction,
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

        #[test]
        fn new_grains_should_come_from_center_of_channels() {
            let mut synth = GranularSynthesizer::new();
            synth
                .set_buffer(Arc::new(vec![1.0; 1024]))
                .set_envelope(crate::EnvelopeType::All1)
                .set_grain_initialization_delay(Duration::ZERO)
                .set_num_channels(250);

            let frame = synth.next_frame();

            assert_eq!(*frame.first().unwrap(), 0.0);
            assert_eq!(*frame.last().unwrap(), 0.0);
            assert_eq!(frame[frame.len() / 2], 1.0)
        }
    }

    #[cfg(test)]
    mod grain_initialization_delay {
        use std::{sync::Arc, time::Duration};

        use crate::{
            granular_synthesizer::GranularSynthesizer,
            granular_synthesizer::GranularSynthesizerAction,
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

    #[cfg(test)]
    mod set_buffer {
        use std::sync::Arc;

        use crate::{
            granular_synthesizer::GranularSynthesizer,
            granular_synthesizer::GranularSynthesizerAction,
        };

        #[test]
        fn it_should_allow_setting_a_smaller_buffer_after_a_big_one() {
            let big_buffer = vec![0.0; 1024];
            let small_buffer = vec![0.0; 1];

            let mut synth = GranularSynthesizer::new();
            synth.set_buffer(Arc::new(big_buffer)).set_num_channels(100);

            for _ in 0..1000 {
                synth.next_frame();
            }

            synth
                .set_buffer(Arc::new(small_buffer))
                .set_num_channels(100);

            for _ in 0..1000 {
                synth.next_frame();
            }
        }
    }
}
