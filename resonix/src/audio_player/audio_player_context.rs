use std::sync::Arc;

use cpal::{Device, Host, SampleFormat, StreamConfig};

use crate::FromContext;

/// Contains all the created audio configuration data
pub struct AudioPlayerContext<UserData> {
    pub host: Host,
    pub device: Device,
    pub sample_format: SampleFormat,
    pub stream_config: StreamConfig,
    /// This is arbitrary user-specified data that the user can associate with
    /// the audio context, making it easily retrievable by
    /// implementing FromContext for U
    pub user_data: UserData,
}

impl<D> FromContext<D> for Arc<AudioPlayerContext<D>> {
    fn from_context(context: Arc<AudioPlayerContext<D>>) -> Self {
        Arc::clone(&context)
    }
}

pub type AudioPlayerContextArg<D = ()> = Arc<AudioPlayerContext<D>>;
