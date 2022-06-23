use crate::audio::buffer_handle::BufferHandle;
use crate::audio::buffer_selection_action::BufferSelectionAction;
use crate::audio::buffer_selection_handle::BufferSelectionHandle;
use crate::audio::density_action::DensityAction;
use crate::audio::density_handle::DensityHandle;
use crate::audio::gain_action::GainAction;
use crate::audio::gain_handle::GainHandle;
use crate::audio::global_defaults::FALLBACK_SAMPLE_RATE;
use crate::audio::granular_synthesizer_handle::GranularSynthesizerHandle;
use crate::audio::play_status_action::PlayStatusAction;
use crate::audio::play_status_handle::PlayStatusHandle;
use crate::audio::stream_handle::StreamHandle;
use crate::components::buffer_sample_bars::get_buffer_maxes;
use crate::state::app_action::AppAction;
use common::granular_synthesizer_action::GranularSynthesizerAction;
use std::rc::Rc;
use std::sync::Arc;
use yew::Reducible;

pub type SampleRate = u32;

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

    /// A list with a set length of max amplitudes from the original audio buffer
    /// this makes re-rendering the audio buffer visualization O(1) instead of O(n),
    /// where n is the length of buffer samples.
    ///
    /// The audio amlitudes range from 0.0 -> 100.0 and are formatted as strings to
    /// the tens decimal place.
    pub buffer_maxes: Vec<String>,

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
    ///
    /// but this rate should be updated at audio initialization time.
    pub sample_rate: SampleRate,

    /// Corresponds to the percentage of channels that will output samples
    /// from the `GranularSynthesizer` on every frame (0.0 -> 1.0)
    pub density_handle: DensityHandle,
}

impl Default for AppState {
    fn default() -> Self {
        // Set up a buffer with some silent data and a granular synthesizer for now.
        // Audio context can't be setup until the user interacts with a UI element.
        let buffer_handle = BufferHandle::new_with_silent_buffer();
        let granular_synthesizer_handle =
            GranularSynthesizerHandle::new_with_app_defaults(buffer_handle.get_data(), 48000);

        Self {
            buffer_maxes: Default::default(),
            stream_handle: Default::default(),
            buffer_selection_handle: Default::default(),
            gain_handle: Default::default(),
            play_status_handle: Default::default(),
            audio_initialized: Default::default(),
            audio_loading: Default::default(),
            density_handle: Default::default(),

            // non-default implementations
            sample_rate: FALLBACK_SAMPLE_RATE,
            buffer_handle: buffer_handle.clone(),
            granular_synthesizer_handle,
        }
    }
}

impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next_state = (*self).clone();
        {
            let action = action.clone();
            match action {
                AppAction::SetBuffer(buffer) => {
                    next_state.buffer_maxes = get_buffer_maxes(&buffer);
                    next_state
                        .granular_synthesizer_handle
                        .set_buffer(Arc::clone(&buffer));
                    next_state.buffer_handle = BufferHandle::new(buffer);
                }
                AppAction::SetStreamHandle(stream_handle) => {
                    next_state.stream_handle = stream_handle;
                }
                AppAction::SetBufferSelectionStart(start) => {
                    next_state.buffer_selection_handle.set_mouse_start(start);
                }
                AppAction::SetBufferSelectionEnd(end) => {
                    next_state.buffer_selection_handle.set_mouse_end(end);
                }
                AppAction::SetBufferSelectionMouseDown(mouse_down) => {
                    next_state
                        .buffer_selection_handle
                        .set_mouse_down(mouse_down);
                }
                AppAction::SetGain(gain) => {
                    next_state.gain_handle.set(gain);
                }
                AppAction::SetPlayStatus(play_status) => {
                    next_state.play_status_handle.set(play_status);
                }
                AppAction::SetAudioInitialized(is_initialized) => {
                    next_state.audio_initialized = is_initialized;
                }
                AppAction::SetAudioLoading(loading) => {
                    next_state.audio_loading = loading;
                }
                AppAction::SetSampleRate(sample_rate) => {
                    next_state.sample_rate = sample_rate;

                    next_state
                        .granular_synthesizer_handle
                        .set_sample_rate(sample_rate);
                }
                AppAction::SetDensity(density) => {
                    next_state.density_handle.set(density);
                    next_state.granular_synthesizer_handle.set_density(density);
                }
            }
        }

        Rc::new(next_state)
    }
}
