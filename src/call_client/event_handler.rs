#![allow(unused_variables)]

use pyo3::prelude::*;
use pyo3::types::PyTuple;

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
    // Since this is a base class it might be that subclasses have constructor
    // parameters. Constructor arguments would be passed to new() even if we
    // don't really need them. So, in order to accept any subclass arguments we
    // just use a *args extra positional arguments trick.
    #[new]
    #[pyo3(signature = (*args))]
    fn new(args: &Bound<'_, PyTuple>) -> PyResult<Self> {
        Ok(Self {})
    }

    /// Event emitted when the active speaker of the call has changed.
    ///
    /// :param dict participant: See :ref:`Participant`
    fn on_active_speaker_changed(&self, participant: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a custom app message is received from another
    /// participant or via the REST API.
    ///
    /// :param string message: Message received from a remote participant
    /// :param string sender: Sender of the message
    fn on_app_message(&self, message: PyObject, sender: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when an audio device is plugged or removed.
    ///
    /// :param dict available_devices: See :ref:`AvailableDevices`
    fn on_available_devices_updated(&self, available_devices: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the call state changes, normally as a consequence of
    /// invocations to :func:`daily.CallClient.join` or
    /// :func:`daily.CallClient.leave`
    ///
    /// :param string state: See :ref:`CallState`
    fn on_call_state_updated(&self, state: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when dial-in is ready. This happens after the room has
    /// connected to the SIP endpoint and the system is ready to receive dial-in
    /// calls.
    ///
    /// :param string sip_endpoint: The SIP endpoint the room has connected to
    fn on_dialin_ready(&self, sip_endpoint: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the session with the dial-out remote end is
    /// answered.
    ///
    /// :param Mapping[str, Any] data: See :ref:`DialoutEvent`
    fn on_dialout_answered(&self, data: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the session with the dial-out remote end is
    /// established.
    ///
    /// :param Mapping[str, Any] data: See :ref:`DialoutEvent`
    fn on_dialout_connected(&self, data: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted in the case of dial-out errors which are fatal and the
    /// service cannot proceed. For example, an error in SDP negotiation is
    /// fatal to the media/SIP pipeline and will result in dialout-error being
    /// triggered.
    ///
    /// :param Mapping[str, Any] data: See :ref:`DialoutEvent`
    fn on_dialout_error(&self, data: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the dial-out remote end disconnects the call or the
    /// call is stopped by calling :func:`daily.CallClient.stop_dialout`.
    ///
    /// :param Mapping[str, Any] data: See :ref:`DialoutEvent`
    fn on_dialout_stopped(&self, data: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted there is a dial-out non-fatal error, such as the selected
    /// codec not being used and a fallback codec being utilized.
    ///
    /// :param Mapping[str, Any] data: See :ref:`DialoutEvent`
    fn on_dialout_warning(&self, data: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when an error occurs.
    ///
    /// :param string message: The error message
    fn on_error(&self, message: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the input settings are updated, normally as a
    /// consequence of invocations to :func:`daily.CallClient.join`,
    /// :func:`daily.CallClient.leave` or
    /// :func:`daily.CallClient.update_inputs`.
    ///
    /// :param dict inputs: See :ref:`InputSettings`
    fn on_inputs_updated(&self, input_settings: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a live stream encounters an
    /// error.
    ///
    /// :param string stream_id: The ID of the live stream that generated the error
    /// :param string message: The error message
    fn on_live_stream_error(&self, stream_id: PyObject, message: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a live stream starts.
    ///
    /// :param dict status: See :ref:`LiveStreamStatus`
    fn on_live_stream_started(&self, status: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a live stream stops.
    ///
    /// :param string stream_id: The ID of the live stream that was stopped
    fn on_live_stream_stopped(&self, stream_id: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a live stream is updated.
    ///
    /// :param Mapping[str, Any] update: See :ref:`LiveStreamUpdate`
    fn on_live_stream_updated(&self, update: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a live stream encounters a
    /// warning.
    ///
    /// :param string stream_id: The ID of the live stream that generated the warning
    /// :param string message: The warning message
    fn on_live_stream_warning(&self, stream_id: PyObject, message: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the logging & telemetry backend updates the network
    /// statistics.
    ///
    /// :param dict stats: See :ref:`NetworkStats`
    fn on_network_stats_updated(&self, stats: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the participant count changes.
    ///
    /// :param dict stats: See :ref:`ParticipantCounts`
    fn on_participant_counts_updated(&self, counts: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a participant joins the call.
    ///
    /// :param dict participant: See :ref:`Participant`
    fn on_participant_joined(&self, participant: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a participant has left the call.
    ///
    /// :param dict participant: See :ref:`Participant`
    /// :param string reason: See :ref:`ParticipantLeftReason`
    fn on_participant_left(&self, participant: PyObject, reason: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a participant is updated. This can mean either the
    /// participant's metadata was updated, or the tracks belonging to the
    /// participant changed.
    ///
    /// :param dict participant: See :ref:`Participant`
    fn on_participant_updated(&self, participant: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the publishing settings are updated, normally as a
    /// consequence of invocations to :func:`daily.CallClient.join`,
    /// :func:`daily.CallClient.update_publishing`.
    ///
    /// :param dict publishing_settings: See :ref:`PublishingSettings`
    fn on_publishing_updated(&self, publishing_settings: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a recording error occurs.
    ///
    /// :param string stream_id: The ID of the recording that generated the error
    /// :param string message: The error message
    fn on_recording_error(&self, stream_id: PyObject, message: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a recording starts.
    ///
    /// :param dict status: See :ref:`RecordingStatus`
    fn on_recording_started(&self, status: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a recording stops.
    ///
    /// :param string stream_id: The ID of the live stream that was stopped
    fn on_recording_stopped(&self, stream_id: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the subscription profile settings are updated as a
    /// consequence of calls to
    /// :func:`daily.CallClient.update_subscription_profiles`.
    ///
    /// :param dict subscription_profiles: See :ref:`SubscriptionProfileSettings`
    fn on_subscription_profiles_updated(&self, subscription_profiles: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the subscription settings are updated as a
    /// consequence of calls to :func:`daily.CallClient.update_subscriptions`.
    ///
    /// :param dict subscriptions: See :ref:`ParticipantSubscriptions`
    fn on_subscriptions_updated(&self, subscriptions: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a transcription error occurs.
    ///
    /// :param string message: The error message
    fn on_transcription_error(&self, message: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a transcription message is received.
    ///
    /// :param dict message: See :ref:`TranscriptionMessage`
    fn on_transcription_message(&self, message: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when transcription starts.
    ///
    /// :param dict status: See :ref:`TranscriptionStatus`
    fn on_transcription_started(&self, status: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when transcription stops.
    ///
    /// :param str stopped_by: The ID of the participant that stopped the transcription or None
    /// :param str stopped_by_error: Whether the transcription was stopped by an error
    fn on_transcription_stopped(
        &self,
        stopped_by: PyObject,
        stopped_by_error: PyObject,
    ) -> PyResult<()> {
        Ok(())
    }
}
