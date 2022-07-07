use crate::audio::audio_ouput_handle::AudioOutputHandle;
use crate::audio::audio_recorder_handle::AudioRecorderHandle;
use crate::audio::buffer_handle::BufferHandle;
use crate::audio::buffer_selection_handle::BufferSelectionHandle;
use crate::audio::density_handle::DensityHandle;
use crate::audio::gain_handle::GainHandle;
use crate::audio::grain_len_handle::GrainLenHandle;
use crate::audio::granular_synthesizer_handle::GranularSynthesizerHandle;
use crate::audio::play_status_handle::PlayStatusHandle;
use crate::audio::recording_status_handle::RecordingStatusHandle;
use crate::audio::refresh_interval_handle::RefreshIntervalHandle;
use crate::audio::stream_handle::StreamHandle;
use audio_common::granular_synthesizer::GranularSynthesizer;
use audio_common::granular_synthesizer_action::GranularSynthesizerAction;

pub type SampleRate = u32;
pub type NunChannels = u32;

/// Global app-level state.
///
/// There are two approaches that are used to update state in this struct:
///
/// ## Replace
/// This is the default apprach that is used to update UI state.
/// Any updated state is entirely replaced (i.e. not mutated) with an updated struct.
/// This is similar to appraches like Redux reducers.
///
/// ## Update in place
/// This is default approach for state that is accessed from the audio thread.
/// Because the audio thread needs a handle to a stable location in memory,
/// these values cannot be replaced. They must be updated in place. The outer handle is cloned, while
/// the inner memory remains consistent. Then, on any state updates, an internal counter in the handle itself
/// is modified so that Yew can compare the previous handle to the new handle and see that the object was updated.
///
/// If we did not update the handle's internal state in some way, Yew would have no way of comparing
/// previous Handles to new Handles, because the outer Handle would be identical in both, and the internal
/// memory/pointer would also be identical.
#[derive(Clone, Debug, PartialEq)]
pub struct AppState {
    /// The currently loaded audio buffer
    pub buffer_handle: BufferHandle,

    /// A list with a set length of max amplitudes from the original audio buffer.
    /// This makes re-rendering the audio buffer visualization and O(1) operation instead of O(n),
    /// where n is the length of buffer samples.
    ///
    /// The audio amlitudes range from 0.0 -> 1.0
    pub buffer_maxes_for_canvas: Vec<f32>,

    /// A handle to the audio context stream (keeps audio playing & stops audio when dropped)
    pub stream_handle: Option<StreamHandle>,

    /// Represents what portion of the audio buffer is currently selected
    pub buffer_selection_handle: BufferSelectionHandle,

    /// Overall audio gain for output audio
    pub gain_handle: GainHandle,

    /// Current play / pause status
    pub play_status_handle: PlayStatusHandle,

    /// Has audio been initialized yet? Audio interactions must be initiated from
    /// a user touch / interaction.
    pub audio_initialized: bool,

    /// If audio currently being initialized?
    pub audio_loading: bool,

    /// Enables updating GranularSynthesizerData from the UI while also getting
    /// audio frames / mutating internal state from the audio thread.
    pub granular_synthesizer_handle: GranularSynthesizerHandle,

    /// Sample rate is instantiated with a fallback sample rate,
    /// but this rate should be updated at audio initialization time.
    pub sample_rate: SampleRate,

    /// Number of channels is instantiated with a fallback number of channels,
    /// but this should be updated at audio initialization time.
    pub num_channels: NunChannels,

    /// Corresponds to the percentage of channels that will output samples
    /// from the `GranularSynthesizer` on every frame (0.0 -> 1.0)
    pub density_handle: DensityHandle,

    /// This is the maximum length (in milliseconds) that a grain sample can play for
    pub grain_len_max: GrainLenHandle,

    /// This is the minimum length (in milliseconds) that a grain sample can play for
    pub grain_len_min: GrainLenHandle,

    pub refresh_interval: RefreshIntervalHandle,

    pub audio_recorder_handle: AudioRecorderHandle,

    pub recording_status_handle: RecordingStatusHandle,

    /// Contains a buffer of the last few audio frames to give an moving average of channel amplitudes.
    /// This value isn't rendered as a component, but it is used to draw a gain visualization on a canvas.
    pub audio_output_handle: AudioOutputHandle,

    pub is_keyboard_user: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            buffer_handle: Default::default(),
            buffer_maxes_for_canvas: Default::default(),
            stream_handle: Default::default(),
            buffer_selection_handle: Default::default(),
            gain_handle: Default::default(),
            play_status_handle: Default::default(),
            audio_initialized: Default::default(),
            audio_loading: Default::default(),
            sample_rate: Default::default(),
            density_handle: Default::default(),
            granular_synthesizer_handle: Default::default(),
            audio_recorder_handle: Default::default(),
            recording_status_handle: Default::default(),
            num_channels: Default::default(),
            audio_output_handle: Default::default(),
            is_keyboard_user: Default::default(),
            grain_len_min: GranularSynthesizer::GRAIN_LEN_MIN_MIN.into(),
            grain_len_max: GranularSynthesizer::GRAIN_LEN_MAX_MAX.into(),
            refresh_interval: GranularSynthesizer::DEFAULT_REFRESH_INTERVAL.into(),
        }
    }
}
