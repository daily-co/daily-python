use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
#[serde(tag = "preset")]
pub enum LiveStreamEndpoints {
    #[serde(rename = "preconfigured")]
    PreConfigured {
        #[serde(rename = "preConfiguredEndpoints")]
        pre_configured_endpoints: Vec<Value>,
    },
    #[serde(rename = "rtmpUrls")]
    RtmpUrls {
        #[serde(rename = "rtmpUrls")]
        rtmp_urls: Vec<Value>,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartLiveStreamProperties {
    pub endpoints: LiveStreamEndpoints,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming_settings: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_new: Option<bool>,
}
