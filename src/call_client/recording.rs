use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRecordingProperties {
    pub streaming_settings: Option<Value>,
    pub instance_id: Option<String>,
    pub force_new: Option<bool>,
}
