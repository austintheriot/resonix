use std::sync::Arc;

use crate::{DACConfig, DataFromDACConfig};

impl DataFromDACConfig for Arc<DACConfig> {
    fn from_config(config: Arc<DACConfig>) -> Self {
        Arc::clone(&config)
    }
}
