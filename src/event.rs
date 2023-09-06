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
        "active-speaker-changed" => object
            .get("participant")
            .map(|participant| vec![DictValue(participant.clone())]),
        "app-message" => {
            if let Some(from) = object.get("from") {
                object
                    .get("msgData")
                    .map(|message| vec![DictValue(from.clone()), DictValue(message.clone())])
            } else {
                None
            }
        }
        "available-devices-updated" => object
            .get("availableDevices")
            .map(|devices| vec![DictValue(devices.clone())]),
        "call-state-updated" => object
            .get("state")
            .map(|state| vec![DictValue(state.clone())]),

        "error" => object
            .get("message")
            .map(|message| vec![DictValue(message.clone())]),

        "inputs-updated" => object
            .get("inputs")
            .map(|inputs| vec![DictValue(inputs.clone())]),
        "live-stream-error" => {
            if let Some(stream_id) = object.get("streamId") {
                object
                    .get("message")
                    .map(|message| vec![DictValue(stream_id.clone()), DictValue(message.clone())])
            } else {
                None
            }
        }
        "live-stream-started" => object
            .get("status")
            .map(|status| vec![DictValue(status.clone())]),
        "live-stream-stopped" => object
            .get("streamId")
            .map(|stream_id| vec![DictValue(stream_id.clone())]),
        "live-stream-warning" => {
            if let Some(stream_id) = object.get("streamId") {
                object
                    .get("message")
                    .map(|message| vec![DictValue(stream_id.clone()), DictValue(message.clone())])
            } else {
                None
            }
        }
        "network-stats-updated" => Some(vec![DictValue(Value::Object(object.clone()))]),
        "participant-counts-updated" => Some(vec![DictValue(Value::Object(object.clone()))]),
        "participant-joined" => object
            .get("participant")
            .map(|participant| vec![DictValue(participant.clone())]),
        "participant-left" => {
            if let Some(participant) = object.get("participant") {
                object
                    .get("leftReason")
                    .map(|reason| vec![DictValue(participant.clone()), DictValue(reason.clone())])
            } else {
                None
            }
        }
        "participant-updated" => object
            .get("participant")
            .map(|participant| vec![DictValue(participant.clone())]),
        "publishing-updated" => object
            .get("publishing")
            .map(|publishing| vec![DictValue(publishing.clone())]),
        "recording-error" => {
            if let Some(stream_id) = object.get("streamId") {
                object
                    .get("message")
                    .map(|message| vec![DictValue(stream_id.clone()), DictValue(message.clone())])
            } else {
                None
            }
        }
        "recording-started" => object
            .get("status")
            .map(|status| vec![DictValue(status.clone())]),
        "recording-stopped" => object
            .get("streamId")
            .map(|stream_id| vec![DictValue(stream_id.clone())]),
        "subscription-profiles-updated" => object
            .get("profiles")
            .map(|profiles| vec![DictValue(profiles.clone())]),
        "subscriptions-updated" => object
            .get("subscriptions")
            .map(|subscriptions| vec![DictValue(subscriptions.clone())]),
        a => panic!("args for event {a} not supported"),
    }
}
