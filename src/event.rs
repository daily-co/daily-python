use crate::dict::DictValue;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub(crate) struct Event {
    pub action: String,
    #[serde(flatten)]
    pub data: DictValue,
}

pub(crate) fn request_id_from_event(event: &Event) -> Option<u64> {
    if let Some(object) = event.data.0.as_object() {
        if let Some(request_id) = object.get("requestId") {
            if let Some(id) = request_id.get("id") {
                id.as_u64()
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

pub(crate) fn method_name_from_event(event: &Event) -> Option<&str> {
    let method_name = match event.action.as_str() {
        "active-speaker-changed" => "on_active_speaker_changed",
        "app-message" => "on_app_message",
        "available-devices-updated" => "on_available_devices_updated",
        "call-state-updated" => "on_call_state_updated",
        "error" => "on_error",
        "inputs-updated" => "on_inputs_updated",
        "live-stream-error" => "on_live_stream_error",
        "live-stream-started" => "on_live_stream_started",
        "live-stream-stopped" => "on_live_stream_stopped",
        "live-stream-warning" => "on_live_stream_warning",
        "network-stats-updated" => "on_network_stats_updated",
        "participant-counts-updated" => "on_participant_counts_updated",
        "participant-joined" => "on_participant_joined",
        "participant-left" => "on_participant_left",
        "participant-updated" => "on_participant_updated",
        "publishing-updated" => "on_publishing_updated",
        "recording-error" => "on_recording_error",
        "recording-started" => "on_recording_started",
        "recording-stopped" => "on_recording_stopped",
        "subscription-profiles-updated" => "on_subscription_profiles_updated",
        "subscriptions-updated" => "on_subscriptions_updated",
        a => {
            tracing::debug!("unimplemented event handler {a}");
            return None;
        }
    };

    Some(method_name)
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
        "request-completed" => {
            if let Some(request_success) = object.get("requestSuccess") {
                Some(vec![
                    DictValue(request_success.clone()),
                    DictValue(Value::Null),
                ])
            } else if let Some(request_error) = object.get("requestError") {
                request_error
                    .get("msg")
                    .map(|msg| vec![DictValue(Value::Null), DictValue(msg.clone())])
            } else {
                Some(vec![DictValue(Value::Null), DictValue(Value::Null)])
            }
        }
        "subscription-profiles-updated" => object
            .get("profiles")
            .map(|profiles| vec![DictValue(profiles.clone())]),
        "subscriptions-updated" => object
            .get("subscriptions")
            .map(|subscriptions| vec![DictValue(subscriptions.clone())]),
        a => panic!("args for event {a} not supported"),
    }
}
