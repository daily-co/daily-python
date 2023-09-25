use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    ptr,
    str::FromStr,
    sync::{Arc, Mutex},
};

use pyo3::{
    exceptions,
    prelude::*,
    types::{PyBytes, PyTuple},
};

use webrtc_daily::sys::color_format::ColorFormat;

use daily_core::prelude::*;

use crate::{
    dict::DictValue,
    event::{args_from_event, method_name_from_event, request_id_from_event, Event},
    PyVideoFrame, GLOBAL_CONTEXT,
};

#[derive(Clone)]
struct CallClientPtr {
    ptr: *mut CallClient,
}

impl CallClientPtr {
    fn as_mut(&mut self) -> &mut CallClient {
        unsafe { &mut *(self.ptr) }
    }
}

unsafe impl Send for CallClientPtr {}

struct CallbackContext {
    callback: Option<PyObject>,
    completions: Arc<Mutex<HashMap<u64, PyObject>>>,
}

#[derive(Clone)]
struct CallbackContextPtr {
    ptr: *const CallbackContext,
}

unsafe impl Send for CallbackContextPtr {}

/// This class represents a call client. A call client is a participant of a
/// Daily meeting and it can receive audio and video from other participants in
/// the meeting as well as send audio and video. Multiple instances of call
/// clients can be created in the same application.
///
/// :param class event_handler: A subclass of :class:`daily.EventHandler`
#[pyclass(name = "CallClient", module = "daily")]
pub struct PyCallClient {
    call_client: CallClientPtr,
    completions: Arc<Mutex<HashMap<u64, PyObject>>>,
    callback_ctx_ptrs: Vec<CallbackContextPtr>,
}

#[pymethods]
impl PyCallClient {
    /// Create a new call client. The new call client can receive meeting events
    /// through an event handler.
    #[new]
    pub fn new(event_handler: Option<PyObject>) -> PyResult<Self> {
        let call_client = unsafe { daily_core_call_client_create() };
        if !call_client.is_null() {
            let completions = Arc::new(Mutex::new(HashMap::new()));

            let callback_ctx = Arc::new(CallbackContext {
                callback: event_handler,
                completions: completions.clone(),
            });

            let callback_ctx_ptr = Arc::into_raw(callback_ctx);

            let client_delegate = NativeCallClientDelegate::new(
                NativeCallClientDelegatePtr::new(callback_ctx_ptr as *mut libc::c_void),
                NativeCallClientDelegateFns::new(on_event),
            );

            unsafe {
                daily_core_call_client_set_delegate(&mut (*call_client), client_delegate);
            }

            Ok(Self {
                call_client: CallClientPtr { ptr: call_client },
                completions,
                callback_ctx_ptrs: vec![CallbackContextPtr {
                    ptr: callback_ctx_ptr,
                }],
            })
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "unable to create a CallClient() object",
            ))
        }
    }

    /// Join a meeting given the `meeting_url` and the optional `meeting_token`
    /// and `client_settings`. The client settings specifie inputs updates or
    /// publising settings.
    ///
    /// :param str meeting_url: The URL of the Daily meeting to join
    /// :param str meeting_token: Meeting token if needed. This is needed if the client is an owner of the meeting
    /// :param dict client_settings: See :ref:`ClientSettings`
    /// :param func completion: An optional completion callback with two parameters: (:ref:`CallClientJoinData`, :ref:`CallClientError`)
    #[pyo3(signature = (meeting_url, meeting_token = None, client_settings = None, completion = None))]
    pub fn join(
        &mut self,
        meeting_url: &str,
        meeting_token: Option<&str>,
        client_settings: Option<PyObject>,
        completion: Option<PyObject>,
    ) {
        // Meeting URL
        let meeting_url_cstr = CString::new(meeting_url).expect("invalid meeting URL string");

        // Meeting token
        let meeting_token_cstr = meeting_token
            .map(|token| CString::new(token).expect("invalid meeting token string"))
            .or(None);

        // Client settings
        let client_settings_cstr = client_settings
            .map(|settings| {
                let settings_string: String = Python::with_gil(|py| {
                    let settings_map: HashMap<String, DictValue> = settings.extract(py).unwrap();
                    serde_json::to_string(&settings_map).unwrap()
                });
                CString::new(settings_string).expect("invalid client settings string")
            })
            .or(None);

        unsafe {
            let request_id = self.maybe_register_completion(completion);
            daily_core_call_client_join(
                self.call_client.as_mut(),
                request_id,
                meeting_url_cstr.as_ptr(),
                meeting_token_cstr
                    .as_ref()
                    .map_or(ptr::null_mut(), |s| s.as_ptr()),
                client_settings_cstr
                    .as_ref()
                    .map_or(ptr::null_mut(), |s| s.as_ptr()),
            );
        }
    }

    /// Leave a previously joined meeting.
    ///
    /// :param func completion: An optional completion callback with two parameters: (None, :ref:`CallClientError`)
    #[pyo3(signature = (completion = None))]
    pub fn leave(&mut self, completion: Option<PyObject>) {
        let request_id = self.maybe_register_completion(completion);
        unsafe {
            daily_core_call_client_leave(self.call_client.as_mut(), request_id);
        }
    }

    /// Sets this client's user name. The user name is what other participants
    /// might be able to see as a description of this client.
    ///
    /// :param str user_name: This client's user name
    #[pyo3(signature = (user_name))]
    pub fn set_user_name(&mut self, user_name: &str) {
        let user_name_cstr = CString::new(user_name).expect("invalid user name string");

        let request_id = self.maybe_register_completion(None);
        unsafe {
            daily_core_call_client_set_user_name(
                self.call_client.as_mut(),
                request_id,
                user_name_cstr.as_ptr(),
            );
        }
    }

    /// Returns the current participants in the meeting.
    ///
    /// :return: See :ref:`CallAllParticipants`
    /// :rtype: dict
    pub fn participants(&mut self) -> PyResult<PyObject> {
        unsafe {
            let participants_ptr = daily_core_call_client_participants(self.call_client.as_mut());
            let participants_string = CStr::from_ptr(participants_ptr)
                .to_string_lossy()
                .into_owned();

            let participants: HashMap<String, DictValue> =
                serde_json::from_str(participants_string.as_str()).unwrap();

            Ok(Python::with_gil(|py| participants.to_object(py)))
        }
    }

    /// Returns the number of hidden and non-hidden participants in the meeting.
    ///
    /// :return: The number of participants in the meeting. See :ref:`ParticipantCounts`
    /// :rtype: dict
    pub fn participant_counts(&mut self) -> PyResult<PyObject> {
        unsafe {
            let participant_counts_ptr =
                daily_core_call_client_participant_counts(self.call_client.as_mut());
            let participant_counts_string = CStr::from_ptr(participant_counts_ptr)
                .to_string_lossy()
                .into_owned();

            let participant_counts: HashMap<String, DictValue> =
                serde_json::from_str(participant_counts_string.as_str()).unwrap();

            Ok(Python::with_gil(|py| participant_counts.to_object(py)))
        }
    }

    /// Updates remote participants.
    ///
    /// :param dict remote_participants: See :ref:`RemoteParticipantUpdates`
    /// :param func completion: An optional completion callback with two parameters: (None, :ref:`CallClientError`)
    #[pyo3(signature = (remote_participants, completion = None))]
    pub fn update_remote_participants(
        &mut self,
        remote_participants: PyObject,
        completion: Option<PyObject>,
    ) {
        let remote_participants_map: HashMap<String, DictValue> =
            Python::with_gil(|py| remote_participants.extract(py).unwrap());

        let remote_participants_string = serde_json::to_string(&remote_participants_map).unwrap();

        let remote_participants_cstr =
            CString::new(remote_participants_string).expect("invalid remote participants string");

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_update_remote_participants(
                self.call_client.as_mut(),
                request_id,
                remote_participants_cstr.as_ptr(),
            );
        }
    }

    /// Returns the current client inputs. The inputs define the call client
    /// video and audio sources (i.e. cameras and microphones).
    ///
    /// :return: See :ref:`InputSettings`
    /// :rtype: dict
    pub fn inputs(&mut self) -> PyResult<PyObject> {
        unsafe {
            let inputs_ptr = daily_core_call_client_inputs(self.call_client.as_mut());
            let inputs_string = CStr::from_ptr(inputs_ptr).to_string_lossy().into_owned();

            let inputs: HashMap<String, DictValue> =
                serde_json::from_str(inputs_string.as_str()).unwrap();

            Ok(Python::with_gil(|py| inputs.to_object(py)))
        }
    }

    /// Updates input settings. This function allows you to update the call
    /// client video and audio inputs.
    ///
    /// :param dict input_settings: See :ref:`InputSettings`
    /// :param func completion: An optional completion callback with two parameters: (:ref:`InputSettings`, :ref:`CallClientError`)
    #[pyo3(signature = (input_settings, completion = None))]
    pub fn update_inputs(&mut self, input_settings: PyObject, completion: Option<PyObject>) {
        let input_settings_map: HashMap<String, DictValue> =
            Python::with_gil(|py| input_settings.extract(py).unwrap());

        let input_settings_string = serde_json::to_string(&input_settings_map).unwrap();

        let input_settings_cstr =
            CString::new(input_settings_string).expect("invalid input settings string");

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_update_inputs(
                self.call_client.as_mut(),
                request_id,
                input_settings_cstr.as_ptr(),
            );
        }
    }

    /// Returns the current client publishing settings. The publishing settings
    /// specify if media should be published (i.e. sent) and, if so, how it
    /// should be sent (e.g. what resolutions or bitrate).
    ///
    /// :return: See :ref:`PublishingSettings`
    /// :rtype: dict
    pub fn publishing(&mut self) -> PyResult<PyObject> {
        unsafe {
            let publishing_ptr = daily_core_call_client_publishing(self.call_client.as_mut());
            let publishing_string = CStr::from_ptr(publishing_ptr)
                .to_string_lossy()
                .into_owned();

            let publishing: HashMap<String, DictValue> =
                serde_json::from_str(publishing_string.as_str()).unwrap();

            Ok(Python::with_gil(|py| publishing.to_object(py)))
        }
    }

    /// Updates publishing settings. This function allows you to update the call
    /// client video and audio publishing settings.
    ///
    /// :param dict publishing_settings: See :ref:`PublishingSettings`
    /// :param func completion: An optional completion callback with two parameters: (:ref:`PublishingSettings`, :ref:`CallClientError`)
    #[pyo3(signature = (publishing_settings, completion = None))]
    pub fn update_publishing(
        &mut self,
        publishing_settings: PyObject,
        completion: Option<PyObject>,
    ) {
        let publishing_settings_map: HashMap<String, DictValue> =
            Python::with_gil(|py| publishing_settings.extract(py).unwrap());

        let publishing_settings_string = serde_json::to_string(&publishing_settings_map).unwrap();

        let publishing_settings_cstr =
            CString::new(publishing_settings_string).expect("invalid publishing settings string");

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_update_publishing(
                self.call_client.as_mut(),
                request_id,
                publishing_settings_cstr.as_ptr(),
            );
        }
    }

    /// Returns the current client subscriptions. The client subscriptions is a
    /// dictionary containing specific subscriptions per remote participant.
    ///
    /// :return: See :ref:`ParticipantSubscriptions`
    /// :rtype: dict
    pub fn subscriptions(&mut self) -> PyResult<PyObject> {
        unsafe {
            let subscriptions_ptr = daily_core_call_client_subscriptions(self.call_client.as_mut());
            let subscriptions_string = CStr::from_ptr(subscriptions_ptr)
                .to_string_lossy()
                .into_owned();

            let subscriptions: HashMap<String, DictValue> =
                serde_json::from_str(subscriptions_string.as_str()).unwrap();

            Ok(Python::with_gil(|py| subscriptions.to_object(py)))
        }
    }

    /// Updates subscriptions and subscription profiles. This function allows
    /// you to update subscription profiles and at the same time assign specific
    /// subscription profiles to a participant and even change specific settings
    /// for some participants.
    ///
    /// :param dict participant_settings: See :ref:`ParticipantSubscriptions`
    /// :param dict profile_settings: See :ref:`SubscriptionProfileSettings`
    /// :param func completion: An optional completion callback with two parameters: (:ref:`ParticipantSubscriptions`, :ref:`CallClientError`)
    #[pyo3(signature = (participant_settings = None, profile_settings = None, completion = None))]
    pub fn update_subscriptions(
        &mut self,
        participant_settings: Option<PyObject>,
        profile_settings: Option<PyObject>,
        completion: Option<PyObject>,
    ) {
        // Participant subscription settings
        let participant_settings_cstr = participant_settings
            .map(|settings| {
                let settings_map: HashMap<String, DictValue> =
                    Python::with_gil(|py| settings.extract(py).unwrap());
                let settings_string = serde_json::to_string(&settings_map).unwrap();
                CString::new(settings_string).expect("invalid participant settings string")
            })
            .or(None);

        // Profile settings
        let profile_settings_cstr = profile_settings
            .map(|settings| {
                let settings_map: HashMap<String, DictValue> =
                    Python::with_gil(|py| settings.extract(py).unwrap());
                let settings_string = serde_json::to_string(&settings_map).unwrap();
                CString::new(settings_string).expect("invalid profile settings string")
            })
            .or(None);

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_update_subscriptions(
                self.call_client.as_mut(),
                request_id,
                participant_settings_cstr
                    .as_ref()
                    .map_or(ptr::null(), |s| s.as_ptr()),
                profile_settings_cstr
                    .as_ref()
                    .map_or(ptr::null(), |s| s.as_ptr()),
            );
        }
    }

    /// Returns the current client subscription profiles. A subscription profile
    /// gives a set of subscription media settings a name.
    ///
    /// :return: See :ref:`SubscriptionProfileSettings`
    /// :rtype: dict
    pub fn subscription_profiles(&mut self) -> PyResult<PyObject> {
        unsafe {
            let profiles_ptr =
                daily_core_call_client_subscription_profiles(self.call_client.as_mut());
            let profiles_string = CStr::from_ptr(profiles_ptr).to_string_lossy().into_owned();

            let profiles: HashMap<String, DictValue> =
                serde_json::from_str(profiles_string.as_str()).unwrap();

            Ok(Python::with_gil(|py| profiles.to_object(py)))
        }
    }

    /// Updates subscription profiles.
    ///
    /// :param dict profile_settings: See :ref:`SubscriptionProfileSettings`
    /// :param func completion: An optional completion callback with two parameters: (:ref:`SubscriptionProfileSettings`, :ref:`CallClientError`)
    #[pyo3(signature = (profile_settings, completion = None))]
    pub fn update_subscription_profiles(
        &mut self,
        profile_settings: PyObject,
        completion: Option<PyObject>,
    ) {
        let profile_settings_map: HashMap<String, DictValue> =
            Python::with_gil(|py| profile_settings.extract(py).unwrap());

        let profile_settings_string = serde_json::to_string(&profile_settings_map).unwrap();
        let profile_settings_cstr =
            CString::new(profile_settings_string).expect("invalid profile settings string");

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_update_subscription_profiles(
                self.call_client.as_mut(),
                request_id,
                profile_settings_cstr.as_ptr(),
            );
        }
    }

    /// Updates the client permissions. This will only update permissions for
    /// this client and is only allowed if this client is the owner of the
    /// meeting.
    ///
    /// :param dict permissions: See :ref:`ParticipantPermissions`
    /// :param func completion: An optional completion callback with two parameters: (None, :ref:`CallClientError`)
    #[pyo3(signature = (permissions, completion = None))]
    pub fn update_permissions(&mut self, permissions: PyObject, completion: Option<PyObject>) {
        let permissions_map: HashMap<String, DictValue> =
            Python::with_gil(|py| permissions.extract(py).unwrap());

        let permissions_string = serde_json::to_string(&permissions_map).unwrap();
        let permissions_cstr =
            CString::new(permissions_string).expect("invalid permissions string");

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_update_permissions(
                self.call_client.as_mut(),
                request_id,
                permissions_cstr.as_ptr(),
            );
        }
    }

    /// Registers a video renderer for the given video source of the provided
    /// participant. The color format of the received frames can be chosen.
    ///
    /// :param str participant_id: The ID of the participant to receive video from
    /// :param function callback: A function or class method to be called on every received frame. It receives two arguments: the participant ID and a :class:`VideoFrame`
    /// :param str video_source: The video source of the remote participant to receive (e.g. `camera`, `screenVideo` or a custom track name)
    /// :param str color_format: The color format that frames should be received. See :ref:`ColorFormat`
    #[pyo3(signature = (participant_id, callback, video_source = "camera", color_format = "RGBA"))]
    pub fn set_video_renderer(
        &mut self,
        participant_id: &str,
        callback: PyObject,
        video_source: &str,
        color_format: &str,
    ) -> PyResult<()> {
        let participant_cstr = CString::new(participant_id).expect("invalid participant ID string");
        let video_source_cstr = CString::new(video_source).expect("invalid video source string");
        let color_format_cstr = CString::new(color_format).expect("invalid color format string");

        if ColorFormat::from_str(color_format).is_err() {
            return Err(exceptions::PyValueError::new_err(format!(
                "invalid color format '{color_format}'"
            )));
        }

        let callback_ctx = Arc::new(CallbackContext {
            callback: Some(callback),
            completions: self.completions.clone(),
        });

        let callback_ctx_ptr = Arc::into_raw(callback_ctx);

        self.callback_ctx_ptrs.push(CallbackContextPtr {
            ptr: callback_ctx_ptr,
        });

        let video_renderer_delegate = NativeVideoRendererDelegate::new(
            NativeVideoRendererDelegatePtr::new(callback_ctx_ptr as *mut libc::c_void),
            NativeVideoRendererDelegateFns::new(on_video_frame),
        );

        let request_id = self.maybe_register_completion(None);

        unsafe {
            daily_core_call_client_set_participant_video_renderer(
                self.call_client.as_mut(),
                request_id,
                participant_cstr.as_ptr(),
                video_source_cstr.as_ptr(),
                color_format_cstr.as_ptr(),
                video_renderer_delegate,
            );
        }

        Ok(())
    }

    /// Sends a message to other participants, or another specific participant,
    /// during the call.
    ///
    /// :param any message: The message to send (should be serializable to JSON)
    /// :param str participant: The participant to send the message to. Or `None` to broadcast the message
    /// :param func completion: An optional completion callback with two parameters: (None, :ref:`CallClientError`)
    #[pyo3(signature = (message, participant = None , completion = None))]
    pub fn send_app_message(
        &mut self,
        message: PyObject,
        participant: Option<&str>,
        completion: Option<PyObject>,
    ) {
        let message_value: DictValue = Python::with_gil(|py| message.extract(py).unwrap());
        let message_string = serde_json::to_string(&message_value.0).unwrap();
        let message_cstr = CString::new(message_string).expect("invalid message string");

        let participant_cstr = participant
            .map(|p| CString::new(p).expect("invalid participant string"))
            .or(None);

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_send_app_message(
                self.call_client.as_mut(),
                request_id,
                message_cstr.as_ptr(),
                participant_cstr
                    .as_ref()
                    .map_or(ptr::null(), |s| s.as_ptr()),
            );
        }
    }

    /// Returns the latest network statistics.
    ///
    /// :return: See :ref:`NetworkStats`
    /// :rtype: dict
    pub fn get_network_stats(&mut self) -> PyResult<PyObject> {
        unsafe {
            let stats_ptr = daily_core_call_client_get_network_stats(self.call_client.as_mut());
            let stats_string = CStr::from_ptr(stats_ptr).to_string_lossy().into_owned();

            let stats: HashMap<String, DictValue> =
                serde_json::from_str(stats_string.as_str()).unwrap();

            Ok(Python::with_gil(|py| stats.to_object(py)))
        }
    }

    fn maybe_register_completion(&mut self, completion: Option<PyObject>) -> u64 {
        let request_id = unsafe { GLOBAL_CONTEXT.as_ref().unwrap().next_request_id() };

        if let Some(completion) = completion {
            self.completions
                .lock()
                .unwrap()
                .insert(request_id, completion);
        }

        request_id
    }
}

impl Drop for PyCallClient {
    fn drop(&mut self) {
        // This assumes the client has left the meeting.
        unsafe {
            daily_core_call_client_destroy(self.call_client.ptr);
        }

        // Cleanup callback contexts. The callback contexts still have one
        // reference count (because of we drop it but increase it again every
        // time a callback happens). After the client is destroyed it is safe to
        // simply get rid of all of them.
        for callback_ctx_ptr in self.callback_ctx_ptrs.iter() {
            let _callback_ctx = unsafe { Arc::from_raw(callback_ctx_ptr.ptr) };
            // This will properly drop the CallbackContext.
        }
    }
}

unsafe extern "C" fn on_event(
    delegate: *mut libc::c_void,
    event_json: *const libc::c_char,
    _json_len: isize,
) {
    Python::with_gil(|py| {
        let callback_ctx_ptr = delegate as *const CallbackContext;

        // We increment the reference count because otherwise it will get
        // dropped when Arc::from_raw() takes ownership, and we still want to
        // keep the delegate pointer around.
        Arc::increment_strong_count(callback_ctx_ptr);

        let callback_ctx = Arc::from_raw(callback_ctx_ptr);

        let event_string = CStr::from_ptr(event_json).to_string_lossy().into_owned();

        let event = serde_json::from_str::<Event>(event_string.as_str()).unwrap();

        match event.action.as_str() {
            "request-completed" => {
                if let Some(request_id) = request_id_from_event(&event) {
                    if let Some(callback) =
                        callback_ctx.completions.lock().unwrap().remove(&request_id)
                    {
                        if let Some(args) = args_from_event(&event) {
                            let py_args = PyTuple::new(py, args);

                            if let Err(error) = callback.call1(py, py_args) {
                                error.write_unraisable(py, None);
                            }
                        }
                    }
                }
            }
            _ => {
                if let Some(callback) = &callback_ctx.callback {
                    if let Some(method_name) = method_name_from_event(&event) {
                        if let Some(args) = args_from_event(&event) {
                            let py_args = PyTuple::new(py, args);

                            if let Err(error) = callback.call_method1(py, method_name, py_args) {
                                error.write_unraisable(py, None);
                            }
                        }
                    }
                }
            }
        }
    });
}

unsafe extern "C" fn on_video_frame(
    delegate: *mut libc::c_void,
    peer_id: *const libc::c_char,
    frame: *const NativeVideoFrame,
) {
    Python::with_gil(|py| {
        let callback_ctx_ptr = delegate as *const CallbackContext;

        // We increment the reference count because otherwise it will get
        // dropped when Arc::from_raw() takes ownership, and we still want to
        // keep the delegate pointer around.
        Arc::increment_strong_count(callback_ctx_ptr);

        let callback_ctx = Arc::from_raw(callback_ctx_ptr);

        if let Some(callback) = &callback_ctx.callback {
            let peer_id = CStr::from_ptr(peer_id).to_string_lossy().into_owned();

            let color_format = CStr::from_ptr((*frame).color_format)
                .to_string_lossy()
                .into_owned();

            let video_frame = PyVideoFrame {
                buffer: PyBytes::from_ptr(py, (*frame).buffer, (*frame).buffer_size).into_py(py),
                width: (*frame).width,
                height: (*frame).height,
                timestamp_us: (*frame).timestamp_us,
                color_format: color_format.into_py(py),
            };

            let args = PyTuple::new(py, &[peer_id.into_py(py), video_frame.into_py(py)]);

            if let Err(error) = callback.call1(py, args) {
                error.write_unraisable(py, None);
            }
        }
    });
}
