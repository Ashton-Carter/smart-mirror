
#[derive(Clone)]
pub struct AppState {
    pub messages: std::sync::Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
}