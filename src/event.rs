use crate::dict::DictValue;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub(crate) struct Event {
    pub action: String,
    #[serde(flatten)]
    pub data: DictValue,
}

pub(crate) fn method_name_from_event(event: &Event) -> Option<&str> {
    match event.action.as_str() {
        "active-speaker-changed" => Some("on_active_speaker_changed"),
        "app-message" => Some("on_app_message"),
        "available-devices-updated" => Some("on_available_devices_updated"),
        "call-state-updated" => Some("on_call_state_updated"),
        "error" => Some("on_error"),
        "inputs-updated" => Some("on_inputs_updated"),
        "live-stream-error" => Some("on_live_stream_error"),
        "live-stream-started" => Some("on_live_stream_started"),
        "live-stream-stopped" => Some("on_live_stream_stopped"),
        "live-stream-warning" => Some("on_live_stream_warning"),
        "network-stats-updated" => Some("on_network_stats_updated"),
        "participant-counts-updated" => Some("on_participant_counts_updated"),
        "participant-joined" => Some("on_participant_joined"),
        "participant-left" => Some("on_participant_left"),
        "participant-updated" => Some("on_participant_updated"),
        "publishing-updated" => Some("on_publishing_updated"),
        "recording-error" => Some("on_recording_error"),
        "recording-started" => Some("on_recording_started"),
        "recording-stopped" => Some("on_recording_stopped"),
        "subscription-profiles-updated" => Some("on_subscription_profiles_updated"),
        "subscriptions-updated" => Some("on_subscriptions_updated"),
        a => {
            tracing::debug!("unimplemented event handler {a}");
            None
        }
    }
}

pub(crate) fn args_from_event(event: &Event) -> Option<Vec<DictValue>> {
    let object = event.data.0.as_object().unwrap();
    match event.action.as_str() {
        "active-speaker-changed" => {
            if let Some(participant) = object.get("participant") {
                Some(vec![DictValue(participant.clone())])
            } else {
                None
            }
        }
        "app-message" => {
            if let Some(from) = object.get("from") {
                if let Some(message) = object.get("msgData") {
                    Some(vec![DictValue(from.clone()), DictValue(message.clone())])
                } else {
                    None
                }
            } else {
                None
            }
        }
        "available-devices-updated" => {
            if let Some(devices) = object.get("availableDevices") {
                Some(vec![DictValue(devices.clone())])
            } else {
                None
            }
        }
        "call-state-updated" => {
            if let Some(state) = object.get("state") {
                Some(vec![DictValue(state.clone())])
            } else {
                None
            }
        }
        "error" => {
            if let Some(message) = object.get("message") {
                Some(vec![DictValue(message.clone())])
            } else {
                None
            }
        }
        "inputs-updated" => {
            if let Some(inputs) = object.get("inputs") {
                Some(vec![DictValue(inputs.clone())])
            } else {
                None
            }
        }
        "live-stream-error" => {
            if let Some(stream_id) = object.get("streamId") {
                if let Some(message) = object.get("message") {
                    Some(vec![
                        DictValue(stream_id.clone()),
                        DictValue(message.clone()),
                    ])
                } else {
                    None
                }
            } else {
                None
            }
        }
        "live-stream-started" => {
            if let Some(status) = object.get("status") {
                Some(vec![DictValue(status.clone())])
            } else {
                None
            }
        }
        "live-stream-stopped" => {
            if let Some(stream_id) = object.get("streamId") {
                Some(vec![DictValue(stream_id.clone())])
            } else {
                None
            }
        }
        "live-stream-warning" => {
            if let Some(stream_id) = object.get("streamId") {
                if let Some(message) = object.get("message") {
                    Some(vec![
                        DictValue(stream_id.clone()),
                        DictValue(message.clone()),
                    ])
                } else {
                    None
                }
            } else {
                None
            }
        }
        "network-stats-updated" => {
            if let Some(inputs) = object.get("inputs") {
                Some(vec![DictValue(inputs.clone())])
            } else {
                None
            }
        }
        "participant-counts-updated" => Some(vec![DictValue(Value::Object(object.clone()))]),
        "participant-joined" => {
            if let Some(participant) = object.get("participant") {
                Some(vec![DictValue(participant.clone())])
            } else {
                None
            }
        }
        "participant-left" => {
            if let Some(participant) = object.get("participant") {
                if let Some(reason) = object.get("leftReason") {
                    Some(vec![
                        DictValue(participant.clone()),
                        DictValue(reason.clone()),
                    ])
                } else {
                    None
                }
            } else {
                None
            }
        }
        "participant-updated" => {
            if let Some(participant) = object.get("participant") {
                Some(vec![DictValue(participant.clone())])
            } else {
                None
            }
        }
        "publishing-updated" => {
            if let Some(publishing) = object.get("publishing") {
                Some(vec![DictValue(publishing.clone())])
            } else {
                None
            }
        }
        "recording-error" => {
            if let Some(stream_id) = object.get("streamId") {
                if let Some(message) = object.get("message") {
                    Some(vec![
                        DictValue(stream_id.clone()),
                        DictValue(message.clone()),
                    ])
                } else {
                    None
                }
            } else {
                None
            }
        }
        "recording-started" => {
            if let Some(status) = object.get("status") {
                Some(vec![DictValue(status.clone())])
            } else {
                None
            }
        }
        "recording-stopped" => {
            if let Some(stream_id) = object.get("streamId") {
                Some(vec![DictValue(stream_id.clone())])
            } else {
                None
            }
        }
        "subscription-profiles-updated" => {
            if let Some(profiles) = object.get("profiles") {
                Some(vec![DictValue(profiles.clone())])
            } else {
                None
            }
        }
        "subscriptions-updated" => {
            if let Some(subscriptions) = object.get("subscriptions") {
                Some(vec![DictValue(subscriptions.clone())])
            } else {
                None
            }
        }
        a => panic!("args for event {a} not supported"),
    }
}
