use resonix::AudioPlayer;

use crate::audio::{play_status::PlayStatus, recording_status::RecordingStatus};
use std::{sync::Arc, time::Duration};

use super::app_state::NunChannels;

pub enum AppAction {
    SetBuffer(Arc<Vec<f32>>),
    SetAudioPlayer(AudioPlayer<()>),
    SetBufferSelectionStart(f32),
    IncrementBufferSelectionStart,
    DecrementBufferSelectionStart,
    IncrementBufferSelectionEnd,
    DecrementBufferSelectionEnd,
    SetBufferSelectionEnd(f32),
    SetBufferSelectionMouseDown(bool),
    SetGain(f32),
    SetPlayStatus(PlayStatus),
    SetAudioInitialized(bool),
    SetAudioLoading(bool),
    SetSampleRate(u32),
    SetNumSynthChannels(usize),
    SetGrainLen(Duration),
    SetGrainInitializationDelay(Duration),
    ResetState,
    SetRecordingStatus(RecordingStatus),
    SetNumChannels(NunChannels),
    DownloadAudio,
    SetIsKeyboardUser,
}
