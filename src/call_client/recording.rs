use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRecordingProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming_settings: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_new: Option<bool>,
}
