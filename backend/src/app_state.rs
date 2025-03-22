use std::sync::Mutex;
use crate::config::Config;

pub struct AppState {
    pub config: Mutex<Config>,
}