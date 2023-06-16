use std::sync::Arc;

use cpal::{Device, Host, SampleFormat, StreamConfig};

use crate::UserDataFromContext;

/// Contains all the created audio configuration data
pub struct AudioOutContext<UserData> {
    pub host: Host,
    pub device: Device,
    pub sample_format: SampleFormat,
    pub stream_config: StreamConfig,
    /// This is arbitrary user-specified data that the user can associate with
    /// the audio context, making it easily retrievable by
    /// implementing UserDataFromContext for U
    pub user_data: UserData,
}

impl<D> UserDataFromContext<D> for Arc<AudioOutContext<D>> {
    fn from_context(context: Arc<AudioOutContext<D>>) -> Self {
        Arc::clone(&context)
    }
}

pub type AudioOutContextArg<D = ()> = Arc<AudioOutContext<D>>;