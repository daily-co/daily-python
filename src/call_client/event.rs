use super::delegate::{DelegateContext, PyCallClientCompletion};

use pythonize::pythonize;
use serde::Deserialize;
use serde_json::Value;

use pyo3::prelude::*;

#[derive(Debug, Deserialize)]
pub(crate) struct Event {
    pub action: String,
    #[serde(flatten)]
    pub data: Value,
}

pub(crate) fn method_name_from_event_action(action: &str) -> Option<&str> {
    let method_name = match action {
        "active-speaker-changed" => "on_active_speaker_changed",
        "app-message" => "on_app_message",
        "available-devices-updated" => "on_available_devices_updated",
        "call-state-updated" => "on_call_state_updated",
        "dialin-connected" => "on_dialin_connected",
        "dialin-ready" => "on_dialin_ready",
        "dialin-error" => "on_dialin_error",
        "dialin-stopped" => "on_dialin_stopped",
        "dialin-warning" => "on_dialin_warning",
        "dialout-connected" => "on_dialout_connected",
        "dialout-answered" => "on_dialout_answered",
        "dialout-error" => "on_dialout_error",
        "dialout-stopped" => "on_dialout_stopped",
        "dialout-warning" => "on_dialout_warning",
        "error" => "on_error",
        "inputs-updated" => "on_inputs_updated",
        "live-stream-error" => "on_live_stream_error",
        "live-stream-started" => "on_live_stream_started",
        "live-stream-stopped" => "on_live_stream_stopped",
        "live-stream-updated" => "on_live_stream_updated",
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
        "transcription-error" => "on_transcription_error",
        "transcription-message" => "on_transcription_message",
        "transcription-started" => "on_transcription_started",
        "transcription-stopped" => "on_transcription_stopped",
        "transcription-updated" => "on_transcription_updated",
        a => {
            tracing::debug!("unimplemented event handler {a}");
            return None;
        }
    };

    Some(method_name)
}

pub(crate) fn request_id_from_event(event: &Event) -> Option<u64> {
    if let Some(object) = event.data.as_object() {
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

pub(crate) fn args_from_event(event: &Event) -> Option<Vec<Value>> {
    let object = event.data.as_object().expect("event should be an object");
    match event.action.as_str() {
        "active-speaker-changed" => object
            .get("participant")
            .map(|participant| vec![participant.clone()]),
        "app-message" => {
            if let Some(message) = object.get("msgData") {
                object
                    .get("from")
                    .map(|from| vec![message.clone(), from.clone()])
            } else {
                None
            }
        }
        "available-devices-updated" => object
            .get("availableDevices")
            .map(|devices| vec![devices.clone()]),
        "call-state-updated" => object.get("state").map(|state| vec![state.clone()]),
        "dialin-connected" => Some(vec![Value::Object(object.clone())]),
        "dialin-ready" => object
            .get("sipEndpoint")
            .map(|sip_endpoint| vec![sip_endpoint.clone()]),
        "dialin-error" => Some(vec![Value::Object(object.clone())]),
        "dialin-stopped" => Some(vec![Value::Object(object.clone())]),
        "dialin-warning" => Some(vec![Value::Object(object.clone())]),
        "dialout-connected" => Some(vec![Value::Object(object.clone())]),
        "dialout-answered" => Some(vec![Value::Object(object.clone())]),
        "dialout-error" => Some(vec![Value::Object(object.clone())]),
        "dialout-stopped" => Some(vec![Value::Object(object.clone())]),
        "dialout-warning" => Some(vec![Value::Object(object.clone())]),
        "error" => object.get("message").map(|message| vec![message.clone()]),
        "inputs-updated" => object.get("inputs").map(|inputs| vec![inputs.clone()]),
        "live-stream-error" => {
            if let Some(stream_id) = object.get("streamId") {
                object
                    .get("message")
                    .map(|message| vec![stream_id.clone(), message.clone()])
            } else {
                None
            }
        }
        "live-stream-started" => object.get("status").map(|status| vec![status.clone()]),
        "live-stream-stopped" => object
            .get("streamId")
            .map(|stream_id| vec![stream_id.clone()]),
        "live-stream-updated" => object.get("update").map(|update| vec![update.clone()]),
        "live-stream-warning" => {
            if let Some(stream_id) = object.get("streamId") {
                object
                    .get("message")
                    .map(|message| vec![stream_id.clone(), message.clone()])
            } else {
                None
            }
        }
        "network-stats-updated" => Some(vec![Value::Object(object.clone())]),
        "participant-counts-updated" => Some(vec![Value::Object(object.clone())]),
        "participant-joined" => object
            .get("participant")
            .map(|participant| vec![participant.clone()]),
        "participant-left" => {
            if let Some(participant) = object.get("participant") {
                object
                    .get("leftReason")
                    .map(|reason| vec![participant.clone(), reason.clone()])
            } else {
                None
            }
        }
        "participant-updated" => object
            .get("participant")
            .map(|participant| vec![participant.clone()]),
        "publishing-updated" => object
            .get("publishing")
            .map(|publishing| vec![publishing.clone()]),
        "recording-error" => {
            if let Some(stream_id) = object.get("streamId") {
                object
                    .get("message")
                    .map(|message| vec![stream_id.clone(), message.clone()])
            } else {
                None
            }
        }
        "recording-started" => object.get("status").map(|status| vec![status.clone()]),
        "recording-stopped" => object
            .get("streamId")
            .map(|stream_id| vec![stream_id.clone()]),
        "subscription-profiles-updated" => object
            .get("profiles")
            .map(|profiles| vec![profiles.clone()]),
        "subscriptions-updated" => object
            .get("subscriptions")
            .map(|subscriptions| vec![subscriptions.clone()]),
        "transcription-error" => object.get("message").map(|message| vec![message.clone()]),
        "transcription-message" => Some(vec![Value::Object(object.clone())]),
        "transcription-started" => object.get("status").map(|status| vec![status.clone()]),
        "transcription-stopped" => {
            if let Some(updated_by) = object.get("updatedBy") {
                Some(vec![updated_by.clone(), Value::Bool(false)])
            } else {
                object
                    .get("stoppedByError")
                    .map(|stopped_by_error| vec![Value::Null, stopped_by_error.clone()])
            }
        }
        "transcription-updated" => object.get("update").map(|update| vec![update.clone()]),
        a => panic!("args for event {a} not supported"),
    }
}

pub(crate) fn completion_args_from_event(
    completion: &PyCallClientCompletion,
    event: &Event,
) -> Option<Vec<Value>> {
    let object = event.data.as_object().expect("event should be an object");
    match event.action.as_str() {
        "request-completed" => {
            if let Some(request_success) = object.get("requestSuccess") {
                let args = match completion {
                    PyCallClientCompletion::UnaryFn(_) => {
                        vec![Value::Null]
                    }
                    PyCallClientCompletion::BinaryFn(_) => {
                        vec![request_success.clone(), Value::Null]
                    }
                };
                Some(args)
            } else if let Some(request_error) = object.get("requestError") {
                let args = request_error.get("msg").map(|msg| match completion {
                    PyCallClientCompletion::UnaryFn(_) => {
                        vec![msg.clone()]
                    }
                    PyCallClientCompletion::BinaryFn(_) => {
                        vec![Value::Null, msg.clone()]
                    }
                });
                Some(args.unwrap())
            } else {
                let args = match completion {
                    PyCallClientCompletion::UnaryFn(_) => {
                        vec![Value::Null]
                    }
                    _ => panic!("completion binary functions should have an error or success"),
                };
                Some(args)
            }
        }
        a => panic!("completion args for event {a} not supported"),
    }
}

pub(crate) fn update_inner_values(
    py: Python<'_>,
    delegate_ctx: &DelegateContext,
    event_action: &str,
    args: Vec<Value>,
) {
    match event_action {
        "active-speaker-changed" => {
            let mut active_speaker = delegate_ctx.inner.active_speaker.lock().unwrap();
            *active_speaker = pythonize(py, &args.first()).unwrap().unbind();
        }
        "inputs-updated" => {
            let mut inputs = delegate_ctx.inner.inputs.lock().unwrap();
            *inputs = pythonize(py, &args.first()).unwrap().unbind();
        }
        "network-stats-updated" => {
            let mut network_stats = delegate_ctx.inner.network_stats.lock().unwrap();
            *network_stats = pythonize(py, &args.first()).unwrap().unbind();
        }
        "participant-counts-updated" => {
            let mut participant_counts = delegate_ctx.inner.participant_counts.lock().unwrap();
            *participant_counts = pythonize(py, &args.first()).unwrap().unbind();
        }
        "publishing-updated" => {
            let mut publishing = delegate_ctx.inner.publishing.lock().unwrap();
            *publishing = pythonize(py, &args.first()).unwrap().unbind();
        }
        "subscription-profiles-updated" => {
            let mut profiles = delegate_ctx.inner.subscription_profiles.lock().unwrap();
            *profiles = pythonize(py, &args.first()).unwrap().unbind();
        }
        "subscriptions-updated" => {
            let mut subscriptions = delegate_ctx.inner.subscriptions.lock().unwrap();
            *subscriptions = pythonize(py, &args.first()).unwrap().unbind();
        }
        _ => (),
    }
}
