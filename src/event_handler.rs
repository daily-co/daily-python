#![allow(unused_variables)]

use pyo3::prelude::*;

/// This a base class for event handlers. Event handlers are used to handle
/// events from the meeting, for example when a participant joins or leaves the
/// meeting or when the active speaker changes.
///
/// Event handlers are registered when creating a :class:`daily.CallClient` and
/// should be created as a subclass of this class. Since event handlers are
/// created as a subclass, there is no need implement all the handler methods.
#[derive(Clone, Debug)]
#[pyclass(name = "EventHandler", module = "daily", subclass)]
pub struct PyEventHandler;

#[pymethods]
impl PyEventHandler {
    #[new]
    fn new() -> Self {
        Self {}
    }

    /// Event emitted when the active speaker of the call has changed.
    fn on_active_speaker_changed(&self, participant: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a custom app message is received from another participant.
    fn on_app_message(&self, message: PyObject, from: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when an audio device is plugged or removed from the mobile.
    fn on_available_devices_updated(&self, available_devices: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the call state changes, normally as a consequence of
    /// invocations to :func:`daily.CallClient.join` or
    /// :func:`daily.CallClient.leave`
    fn on_call_state_updated(&self, state: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when an error occurs.
    fn on_error(&self, message: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the input settings are updated, normally as a
    /// consequence of invocations to :func:`daily.CallClient.join`,
    /// :func:`daily.CallClient.leave` or
    /// :func:`daily.CallClient.update_inputs`.
    fn on_inputs_updated(&self, input_settings: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a live stream encounters an
    /// error.
    fn on_live_stream_error(&self, stream_id: PyObject, message: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a live stream starts.
    fn on_live_stream_started(&self, status: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a live stream stops.
    fn on_live_stream_stopped(&self, stream_id: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a live stream encounters a
    /// warning.
    fn on_live_stream_warning(&self, stream_id: PyObject, message: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the logging & telemetry backend updates the network
    /// statistics.
    fn on_network_stats_updated(&self, stats: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the participant count changes.
    fn on_participant_counts_updated(&self, counts: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a participant joins the call.
    fn on_participant_joined(&self, participant: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a participant has left the call.
    fn on_participant_left(&self, participant: PyObject, reason: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a participant is updated. This can mean either the
    /// participant's metadata was updated, or the tracks belonging to the
    /// participant changed.
    fn on_participant_updated(&self, participant: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the publishing settings are updated, normally as a
    /// consequence of invocations to :func:`daily.CallClient.join`,
    /// :func:`daily.CallClient.update_publishing`.
    fn on_publishing_updated(&self, publishing_settings: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a recording error occurs.
    fn on_recording_error(&self, stream_id: PyObject, message: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a recording starts.
    fn on_recording_started(&self, status: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a recording stops.
    fn on_recording_stopped(&self, stream_id: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the subscription profile settings are updated as a
    /// consequence of calls to
    /// :func:`daily.CallClient.update_subscription_profiles`.
    fn on_subscription_profiles_updated(&self, subscription_profiles: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the subscription settings are updated as a
    /// consequence of calls to :func:`daily.CallClient.update_subscriptions`.
    fn on_subscriptions_updated(&self, subscriptions: PyObject) -> PyResult<()> {
        Ok(())
    }
}
