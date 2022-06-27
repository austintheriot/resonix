use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct GlobalEvent<T> {
    pub event: String,
    #[serde(rename(serialize = "windowLabel"))]
    pub window_label: Option<String>,
    pub payload: T,
    pub id: u64,
}


#[derive(Clone, Serialize, Deserialize, Debug, Copy, Hash)]
pub enum AppEventName {
    CounterChanged
}

impl From<AppEventName> for &'static str {
    fn from(app_event: AppEventName) -> Self {
        match app_event {
            AppEventName::CounterChanged => "counter-changed",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum AppPayload {
    NewCount(u32)
}