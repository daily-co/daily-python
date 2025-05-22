pub(crate) mod delegate;
pub(crate) mod event;
pub(crate) mod event_handler;
pub(crate) mod live_stream;
pub(crate) mod recording;

pub(crate) use event_handler::PyEventHandler;
pub(crate) use live_stream::{LiveStreamEndpoints, StartLiveStreamProperties};
pub(crate) use recording::StartRecordingProperties;

use delegate::*;

use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    ptr,
    str::FromStr,
    sync::{Arc, Mutex},
};

use pyo3::{exceptions, prelude::*};
use uuid::Uuid;

use webrtc_daily::sys::color_format::ColorFormat;

use daily_core::prelude::*;

use crate::{util::dict::DictValue, PyCustomAudioTrack, GLOBAL_CONTEXT};

#[derive(Clone)]
struct CallClientPtr {
    ptr: *mut CallClient,
}

impl CallClientPtr {
    fn as_ptr(&mut self) -> *mut CallClient {
        self.ptr
    }

    unsafe fn as_mut(&mut self) -> &mut CallClient {
        &mut *(self.ptr)
    }
}

unsafe impl Send for CallClientPtr {}

/// This class represents a call client. A call client is a participant of a
/// Daily meeting and it can receive audio and video from other participants in
/// the meeting as well as send audio and video. Multiple instances of call
/// clients can be created in the same application.
///
/// :param class event_handler: A subclass of :class:`daily.EventHandler`
#[pyclass(name = "CallClient", module = "daily")]
pub struct PyCallClient {
    call_client: Mutex<Option<CallClientPtr>>,
    inner: Arc<PyCallClientInner>,
    delegate_ctx_ptr: DelegateContextPtr,
}

impl PyCallClient {
    fn check_released(&self) -> PyResult<CallClientPtr> {
        // If we have already been released throw an exception.
        if let Some(call_client) = self.call_client.lock().unwrap().as_ref() {
            Ok(call_client.clone())
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "this object has already been released",
            ))
        }
    }

    fn maybe_register_completion(&self, completion: Option<PyCallClientCompletion>) -> u64 {
        let request_id = GLOBAL_CONTEXT.next_request_id();

        if let Some(completion) = completion {
            self.inner
                .completions
                .lock()
                .unwrap()
                .insert(request_id, completion);
        }

        request_id
    }

    fn start_live_stream(
        &self,
        endpoints: LiveStreamEndpoints,
        streaming_settings: Option<PyObject>,
        stream_id: Option<&str>,
        force_new: Option<bool>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        let mut call_client = self.check_released()?;

        let stream_id = stream_id.map(|id| id.to_string());

        let streaming_settings = streaming_settings.map(|s| {
            let dict: HashMap<String, DictValue> = Python::with_gil(|py| s.extract(py).unwrap());
            dict.iter().map(|(k, v)| (k.clone(), v.0.clone())).collect()
        });

        let properties = StartLiveStreamProperties {
            endpoints,
            streaming_settings,
            stream_id,
            force_new,
        };

        let properties_string = serde_json::to_string(&properties).unwrap();

        let properties_cstr =
            Some(CString::new(properties_string).expect("invalid live stream properties string"));

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_start_live_stream(
                call_client.as_mut(),
                request_id,
                properties_cstr.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }
}

#[pymethods]
impl PyCallClient {
    /// Create a new call client. The new call client can receive meeting events
    /// through an event handler.
    #[new]
    #[pyo3(signature = (event_handler = None))]
    pub fn new(event_handler: Option<PyObject>) -> PyResult<Self> {
        // Make sure the event handler has the right type.
        if let Some(event_handler) = event_handler.clone() {
            let is_event_handler =
                Python::with_gil(|py| event_handler.bind(py).is_instance_of::<PyEventHandler>());

            if !is_event_handler {
                return Err(exceptions::PyTypeError::new_err(
                    "event_handler should be a subclass of `EventHandler`",
                ));
            }
        }

        let call_client = unsafe { daily_core_call_client_create() };
        if !call_client.is_null() {
            // Get initial values
            let active_speaker = unsafe { get_active_speaker(&mut (*call_client))? };
            let inputs = unsafe { get_inputs(&mut (*call_client))? };
            let participant_counts = unsafe { get_participant_counts(&mut (*call_client))? };
            let publishing = unsafe { get_publishing(&mut (*call_client))? };
            let subscriptions = unsafe { get_subscriptions(&mut (*call_client))? };
            let subscription_profiles = unsafe { get_subscription_profiles(&mut (*call_client))? };
            let network_stats = unsafe { get_network_stats(&mut (*call_client))? };

            let inner = Arc::new(PyCallClientInner {
                event_handler_callback: Mutex::new(event_handler),
                delegates: Mutex::new(PyCallClientDelegateFns {
                    on_event: Some(on_event),
                    on_video_frame: Some(on_video_frame),
                    on_audio_data: Some(on_audio_data),
                }),
                completions: Mutex::new(HashMap::new()),
                audio_renderers: Mutex::new(HashMap::new()),
                video_renderers: Mutex::new(HashMap::new()),
                // Non-blocking
                active_speaker: Mutex::new(active_speaker),
                inputs: Mutex::new(inputs),
                participant_counts: Mutex::new(participant_counts),
                publishing: Mutex::new(publishing),
                subscriptions: Mutex::new(subscriptions),
                subscription_profiles: Mutex::new(subscription_profiles),
                network_stats: Mutex::new(network_stats),
            });

            let delegate_ctx = Arc::new(DelegateContext {
                inner: inner.clone(),
            });

            let delegate_ctx_ptr = Arc::into_raw(delegate_ctx);

            let client_delegate = NativeCallClientDelegate::new(
                NativeCallClientDelegatePtr::new(delegate_ctx_ptr as *mut libc::c_void),
                NativeCallClientDelegateFns::new(
                    on_event_native,
                    on_audio_data_native,
                    on_video_frame_native,
                ),
            );

            unsafe {
                daily_core_call_client_set_delegate(&mut (*call_client), client_delegate);
            }

            Ok(Self {
                inner,
                call_client: Mutex::new(Some(CallClientPtr { ptr: call_client })),
                delegate_ctx_ptr: DelegateContextPtr {
                    ptr: delegate_ctx_ptr,
                },
            })
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "unable to create a CallClient() object",
            ))
        }
    }

    /// Release internal resources. This function should be called when this
    /// object is not needed anymore making sure all internal resources are
    /// freed.
    ///
    /// If this function is not called we will attempt to automatically call it
    /// during garbage collection. However, that's not guaranteed (e.g. if
    /// there's a circular dependency with the registered event handler),
    /// therefore it is strongly recommended to always call this function.
    pub fn release(&self, py: Python<'_>) -> PyResult<()> {
        // Hold the call client lock for the whole function so no one else can
        // grab it while we are releasing.
        let mut call_client = self.call_client.lock().unwrap();

        // If we have already been released throw an exception.
        if call_client.is_none() {
            return Err(exceptions::PyRuntimeError::new_err(
                "this object has already been released",
            ));
        }

        {
            // Cleanup video/audio delegates so they are not called during
            // destroy. Do it inside a new scope so the lock gets released.
            //
            // Note that we don't cleanup the event delegate because we might be
            // waiting on completions to finish (e.g. leave).
            let mut delegates = self.inner.delegates.lock().unwrap();
            delegates.on_audio_data.take();
            delegates.on_video_frame.take();
        }

        let mut call_client_cpy = call_client.as_ref().unwrap().clone();

        // Here we release the GIL so we can allow any event delegates to
        // finish. The event delegates will be waiting on the GIL and
        // execute at this point. But since we just cleanup the delegates
        // above, the events will actually be a no-op.
        py.allow_threads(move || unsafe {
            daily_core_call_client_destroy(call_client_cpy.as_ptr());
        });

        // Remove any reference to the Python's event handler. This should get
        // rid of any circular dependency.
        self.inner.event_handler_callback.lock().unwrap().take();

        // Cleanup the delegate context. The delegate context still has one
        // reference count (because of we drop it but increase it again every
        // time a delegate happens). After the client is destroyed it is safe to
        // simply get rid of it.
        let _delegate_ctx = unsafe { Arc::from_raw(self.delegate_ctx_ptr.ptr) };

        // Release the call client pointer. We won't need it anymore.
        *call_client = None;

        Ok(())
    }

    /// Sets a proxy URL for this client. For users whose firewall policies
    /// prevent them from directly accessing Daily’s web domains, using a proxy
    /// URL provide a mechanism to send connections to Daily's HTTPS and
    /// WebSocket endpoints to a specified proxy server instead.
    ///
    /// :param Optional[str] proxy_url: The proxy URL to use or `None` to unset the current proxy.
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (proxy_url = None, completion = None))]
    pub fn set_proxy_url(
        &self,
        proxy_url: Option<&str>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let proxy_url_cstr = proxy_url
            .map(|url| CString::new(url).expect("invalid proxy URL string"))
            .or(None);

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_set_proxy_url(
                call_client.as_mut(),
                request_id,
                proxy_url_cstr
                    .as_ref()
                    .map_or(ptr::null_mut(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Allows for specifying custom TURN servers rather than only using Daily's
    /// default TURN servers.
    ///
    /// :param Optional[Mapping[str, Any]] ice_config: See :ref:`IceConfig` or `None` to unset the current ICE config
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (ice_config = None, completion = None))]
    pub fn set_ice_config(
        &self,
        ice_config: Option<PyObject>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        // Participant subscription settings
        let ice_config_cstr = Python::with_gil(|py| {
            ice_config
                .map(|config| {
                    let config_map: HashMap<String, DictValue> = config.extract(py).unwrap();
                    let config_string = serde_json::to_string(&config_map).unwrap();
                    CString::new(config_string).expect("invalid ICE config string")
                })
                .or(None)
        });

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_set_ice_config(
                call_client.as_mut(),
                request_id,
                ice_config_cstr
                    .as_ref()
                    .map_or(ptr::null_mut(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Join a meeting given the `meeting_url` and the optional `meeting_token`
    /// and `client_settings`. The client settings specifie inputs updates or
    /// publising settings.
    ///
    /// :param str meeting_url: The URL of the Daily meeting to join
    /// :param Optional[str] meeting_token: Meeting token if needed. This is needed if the client is an owner of the meeting
    /// :param Optional[Mapping[str, Any]] client_settings: See :ref:`ClientSettings`
    /// :param Optional[func] completion: An optional completion callback with two parameters: (:ref:`CallClientJoinData`, :ref:`CallClientError`)
    #[pyo3(signature = (meeting_url, meeting_token = None, client_settings = None, completion = None))]
    pub fn join(
        &self,
        meeting_url: &str,
        meeting_token: Option<&str>,
        client_settings: Option<PyObject>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        // Meeting URL
        let meeting_url_cstr = CString::new(meeting_url).expect("invalid meeting URL string");

        // Meeting token
        let meeting_token_cstr = meeting_token
            .map(|token| CString::new(token).expect("invalid meeting token string"))
            .or(None);

        // Client settings
        let client_settings_cstr = Python::with_gil(|py| {
            client_settings
                .map(|settings| {
                    let settings_map: HashMap<String, DictValue> = settings.extract(py).unwrap();
                    let settings_string = serde_json::to_string(&settings_map).unwrap();
                    CString::new(settings_string).expect("invalid client settings string")
                })
                .or(None)
        });

        unsafe {
            let request_id =
                self.maybe_register_completion(completion.map(PyCallClientCompletion::BinaryFn));

            daily_core_call_client_join(
                call_client.as_mut(),
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

        Ok(())
    }

    /// Leave a previously joined meeting.
    ///
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (completion = None))]
    pub fn leave(&self, completion: Option<PyObject>) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_leave(call_client.as_mut(), request_id);
        }

        Ok(())
    }

    /// Sets this client's user name. The user name is what other participants
    /// might be able to see as a description of this client.
    ///
    /// :param str user_name: This client's user name
    pub fn set_user_name(&self, user_name: &str) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let user_name_cstr = CString::new(user_name).expect("invalid user name string");

        let request_id = self.maybe_register_completion(None);
        unsafe {
            daily_core_call_client_set_user_name(
                call_client.as_mut(),
                request_id,
                user_name_cstr.as_ptr(),
            );
        }

        Ok(())
    }

    /// Returns the current active speaker.
    ///
    /// :return: See :ref:`Participant`
    /// :rtype: Mapping[str, Any]
    pub fn active_speaker(&self) -> PyResult<PyObject> {
        // If we have already been released throw an exception.
        self.check_released()?;

        Ok(self.inner.active_speaker.lock().unwrap().clone())
    }

    /// Returns the current participants in the meeting.
    ///
    /// :return: See :ref:`CallParticipants`
    /// :rtype: Mapping[str, Any]
    pub fn participants(&self) -> PyResult<PyObject> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        unsafe {
            let participants_ptr = daily_core_call_client_participants(call_client.as_mut());
            let participants_string = CStr::from_ptr(participants_ptr)
                .to_string_lossy()
                .into_owned();

            let participants: HashMap<String, DictValue> =
                serde_json::from_str(participants_string.as_str()).unwrap();

            Python::with_gil(|py| Ok(participants.to_object(py)))
        }
    }

    /// Returns the number of hidden and non-hidden participants in the meeting.
    ///
    /// :return: The number of participants in the meeting. See :ref:`ParticipantCounts`
    /// :rtype: Mapping[str, Any]
    pub fn participant_counts(&self) -> PyResult<PyObject> {
        // If we have already been released throw an exception.
        self.check_released()?;

        Ok(self.inner.participant_counts.lock().unwrap().clone())
    }

    /// Updates remote participants.
    ///
    /// :param Mapping[str, Any] remote_participants: See :ref:`RemoteParticipantUpdates`
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (remote_participants, completion = None))]
    pub fn update_remote_participants(
        &self,
        remote_participants: PyObject,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let remote_participants_map: HashMap<String, DictValue> =
            Python::with_gil(|py| remote_participants.extract(py).unwrap());

        let remote_participants_string = serde_json::to_string(&remote_participants_map).unwrap();

        let remote_participants_cstr =
            CString::new(remote_participants_string).expect("invalid remote participants string");

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_update_remote_participants(
                call_client.as_mut(),
                request_id,
                remote_participants_cstr.as_ptr(),
            );
        }

        Ok(())
    }

    /// Ejects remote participants.
    ///
    /// :param List[str] ids: A list of IDs of remote participants to eject
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (ids, completion = None))]
    pub fn eject_remote_participants(
        &self,
        ids: PyObject,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let ids: Vec<String> = Python::with_gil(|py| ids.extract(py).unwrap());

        let ids_string = serde_json::to_string(&ids).unwrap();

        let ids_cstr = CString::new(ids_string).expect("invalid participant IDs string");

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_eject_remote_participants(
                call_client.as_mut(),
                request_id,
                ids_cstr.as_ptr(),
            );
        }

        Ok(())
    }

    /// Returns the current client inputs. The inputs define the call client
    /// video and audio sources (i.e. cameras and microphones).
    ///
    /// :return: See :ref:`InputSettings`
    /// :rtype: Mapping[str, Any]
    pub fn inputs(&self) -> PyResult<PyObject> {
        // If we have already been released throw an exception.
        self.check_released()?;

        Ok(self.inner.inputs.lock().unwrap().clone())
    }

    /// Updates input settings. This function allows you to update the call
    /// client video and audio inputs.
    ///
    /// :param Mapping[str, Any] input_settings: See :ref:`InputSettings`
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (input_settings, completion = None))]
    pub fn update_inputs(
        &self,
        input_settings: PyObject,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let input_settings_map: HashMap<String, DictValue> =
            Python::with_gil(|py| input_settings.extract(py).unwrap());

        let input_settings_string = serde_json::to_string(&input_settings_map).unwrap();

        let input_settings_cstr =
            CString::new(input_settings_string).expect("invalid input settings string");

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_update_inputs(
                call_client.as_mut(),
                request_id,
                input_settings_cstr.as_ptr(),
            );
        }

        Ok(())
    }

    /// Adds a new custom audio track with the given name. Audio frames need to
    /// be written using the audio source.
    ///
    /// :param str track_name: The audio track name
    /// :param audio_track: The custom audio track being added
    /// :type audio_track: :class:`daily.CustomAudioTrack`
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (track_name, audio_track, completion = None))]
    pub fn add_custom_audio_track(
        &self,
        track_name: &str,
        audio_track: &PyCustomAudioTrack,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let track_name_cstr = CString::new(track_name).expect("invalid track name string");

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_add_custom_audio_track(
                call_client.as_mut(),
                request_id,
                track_name_cstr.as_ptr(),
                audio_track.audio_track.as_ptr() as *const _,
            );
        }

        Ok(())
    }

    /// Updates an existing custom audio track with a new custom audio
    /// source.
    ///
    /// :param str track_name: The audio track name
    /// :param audio_track: The new custom audio track
    /// :type audio_track: :class:`daily.CustomAudioTrack`
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (track_name, audio_track, completion = None))]
    pub fn update_custom_audio_track(
        &self,
        track_name: &str,
        audio_track: &PyCustomAudioTrack,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let track_name_cstr = CString::new(track_name).expect("invalid track name string");

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_update_custom_audio_track(
                call_client.as_mut(),
                request_id,
                track_name_cstr.as_ptr(),
                audio_track.audio_track.as_ptr() as *const _,
            );
        }

        Ok(())
    }

    /// Removes an existing custom audio track.
    ///
    /// :param str track_name: The audio track name
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (track_name, completion = None))]
    pub fn remove_custom_audio_track(
        &self,
        track_name: &str,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let track_name_cstr = CString::new(track_name).expect("invalid track name string");

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_remove_custom_audio_track(
                call_client.as_mut(),
                request_id,
                track_name_cstr.as_ptr(),
            );
        }

        Ok(())
    }

    /// Returns the current client publishing settings. The publishing settings
    /// specify if media should be published (i.e. sent) and, if so, how it
    /// should be sent (e.g. what resolutions or bitrate).
    ///
    /// :return: See :ref:`PublishingSettings`
    /// :rtype: Mapping[str, Any]
    pub fn publishing(&self) -> PyResult<PyObject> {
        // If we have already been released throw an exception.
        self.check_released()?;

        Ok(self.inner.publishing.lock().unwrap().clone())
    }

    /// Updates publishing settings. This function allows you to update the call
    /// client video and audio publishing settings.
    ///
    /// :param Mapping[str, Any] publishing_settings: See :ref:`PublishingSettings`
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (publishing_settings, completion = None))]
    pub fn update_publishing(
        &self,
        publishing_settings: PyObject,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let publishing_settings_map: HashMap<String, DictValue> =
            Python::with_gil(|py| publishing_settings.extract(py).unwrap());

        let publishing_settings_string = serde_json::to_string(&publishing_settings_map).unwrap();

        let publishing_settings_cstr =
            CString::new(publishing_settings_string).expect("invalid publishing settings string");

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_update_publishing(
                call_client.as_mut(),
                request_id,
                publishing_settings_cstr.as_ptr(),
            );
        }

        Ok(())
    }

    /// Returns the current client subscriptions. The client subscriptions is an
    /// object containing specific subscriptions per remote participant.
    ///
    /// :return: See :ref:`ParticipantSubscriptions`
    /// :rtype: Mapping[str, Any]
    pub fn subscriptions(&self) -> PyResult<PyObject> {
        // If we have already been released throw an exception.
        self.check_released()?;

        Ok(self.inner.subscriptions.lock().unwrap().clone())
    }

    /// Updates subscriptions and subscription profiles. This function allows
    /// you to update subscription profiles and at the same time assign specific
    /// subscription profiles to a participant and even change specific settings
    /// for some participants.
    ///
    /// :param Optional[Mapping[str, Any]] participant_settings: See :ref:`ParticipantSubscriptions`
    /// :param Optional[Mapping[str, Any]] profile_settings: See :ref:`SubscriptionProfileSettings`
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (participant_settings = None, profile_settings = None, completion = None))]
    pub fn update_subscriptions(
        &self,
        participant_settings: Option<PyObject>,
        profile_settings: Option<PyObject>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        // Participant subscription settings
        let participant_settings_cstr = Python::with_gil(|py| {
            participant_settings
                .map(|settings| {
                    let settings_map: HashMap<String, DictValue> = settings.extract(py).unwrap();
                    let settings_string = serde_json::to_string(&settings_map).unwrap();
                    CString::new(settings_string).expect("invalid participant settings string")
                })
                .or(None)
        });

        // Profile settings
        let profile_settings_cstr = Python::with_gil(|py| {
            profile_settings
                .map(|settings| {
                    let settings_map: HashMap<String, DictValue> = settings.extract(py).unwrap();
                    let settings_string = serde_json::to_string(&settings_map).unwrap();
                    CString::new(settings_string).expect("invalid profile settings string")
                })
                .or(None)
        });

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_update_subscriptions(
                call_client.as_mut(),
                request_id,
                participant_settings_cstr
                    .as_ref()
                    .map_or(ptr::null(), |s| s.as_ptr()),
                profile_settings_cstr
                    .as_ref()
                    .map_or(ptr::null(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Returns the current client subscription profiles. A subscription profile
    /// gives a set of subscription media settings a name.
    ///
    /// :return: See :ref:`SubscriptionProfileSettings`
    /// :rtype: Mapping[str, Any]
    pub fn subscription_profiles(&self) -> PyResult<PyObject> {
        // If we have already been released throw an exception.
        self.check_released()?;

        Ok(self.inner.subscription_profiles.lock().unwrap().clone())
    }

    /// Updates subscription profiles.
    ///
    /// :param Mapping[str, Any] profile_settings: See :ref:`SubscriptionProfileSettings`
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (profile_settings, completion = None))]
    pub fn update_subscription_profiles(
        &self,
        profile_settings: PyObject,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let profile_settings_map: HashMap<String, DictValue> =
            Python::with_gil(|py| profile_settings.extract(py).unwrap());

        let profile_settings_string = serde_json::to_string(&profile_settings_map).unwrap();
        let profile_settings_cstr =
            CString::new(profile_settings_string).expect("invalid profile settings string");

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_update_subscription_profiles(
                call_client.as_mut(),
                request_id,
                profile_settings_cstr.as_ptr(),
            );
        }

        Ok(())
    }

    /// Updates the client permissions. This will only update permissions for
    /// this client and is only allowed if this client is the owner of the
    /// meeting.
    ///
    /// :param Mapping[str, Any] permissions: See :ref:`ParticipantPermissions`
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (permissions, completion = None))]
    pub fn update_permissions(
        &self,
        permissions: PyObject,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let permissions_map: HashMap<String, DictValue> =
            Python::with_gil(|py| permissions.extract(py).unwrap());

        let permissions_string = serde_json::to_string(&permissions_map).unwrap();
        let permissions_cstr =
            CString::new(permissions_string).expect("invalid permissions string");

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_update_permissions(
                call_client.as_mut(),
                request_id,
                permissions_cstr.as_ptr(),
            );
        }

        Ok(())
    }

    /// Starts a new live-stream with the given pre-configured endpoints.
    ///
    /// :param List[str] endpoints: A list of preconfigured live streaming endpoints
    /// :param Optional[Mapping[str, Any]] streaming_settings: See :ref:`StreamingSettings`
    /// :param Optional[str] stream_id: A unique stream identifier. Multiple live streaming sessions can be started by specifying a unique ID
    /// :param Optional[str] force_new: Whether to force a new live stream, even if there is already one in progress
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (endpoints, streaming_settings = None, stream_id = None, force_new = None, completion = None))]
    pub fn start_live_stream_with_endpoints(
        &self,
        endpoints: PyObject,
        streaming_settings: Option<PyObject>,
        stream_id: Option<&str>,
        force_new: Option<bool>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        let list: Vec<DictValue> = Python::with_gil(|py| endpoints.extract(py).unwrap());
        let array = list.iter().map(|v| v.0.clone()).collect();
        let endpoints = LiveStreamEndpoints::PreConfigured {
            pre_configured_endpoints: array,
        };

        self.start_live_stream(
            endpoints,
            streaming_settings,
            stream_id,
            force_new,
            completion,
        )
    }

    /// Starts a new live-stream with the given RTMP URLs.
    ///
    /// :param List[str] rtmp_urls: A list of live streaming RTMP URLs
    /// :param Optional[Mapping[str, Any]] streaming_settings: See :ref:`StreamingSettings`
    /// :param Optional[str] stream_id: A unique stream identifier. Multiple live streaming sessions can be started by specifying a unique ID
    /// :param Optional[bool] force_new: Whether to force a new live stream, even if there is already one in progress
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (rtmp_urls, streaming_settings = None, stream_id = None, force_new = None, completion = None))]
    pub fn start_live_stream_with_rtmp_urls(
        &self,
        rtmp_urls: PyObject,
        streaming_settings: Option<PyObject>,
        stream_id: Option<&str>,
        force_new: Option<bool>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        let list: Vec<DictValue> = Python::with_gil(|py| rtmp_urls.extract(py).unwrap());
        let array = list.iter().map(|v| v.0.clone()).collect();
        let endpoints = LiveStreamEndpoints::RtmpUrls { rtmp_urls: array };

        self.start_live_stream(
            endpoints,
            streaming_settings,
            stream_id,
            force_new,
            completion,
        )
    }

    /// Stops an ongoing live stream. If multiple live stream instances are running,
    /// each instance must be stopped individually by providing the unique
    /// stream ID.
    ///
    /// :param Optional[str] stream_id: A unique stream identifier
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (stream_id = None, completion = None))]
    pub fn stop_live_stream(
        &self,
        stream_id: Option<&str>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let stream_id_cstr = stream_id
            .map(|id| CString::new(id).expect("invalid stream id string"))
            .or(None);

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_stop_live_stream(
                call_client.as_mut(),
                request_id,
                stream_id_cstr
                    .as_ref()
                    .map_or(ptr::null_mut(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Updates an ongoing live stream. If multiple live stream instances are
    /// running, each instance must be updated individually by providing the
    /// unique stream ID.
    ///
    /// :param Mapping[str, Any] update_settings: See :ref:`StreamingUpdateSettings`
    /// :param Optional[str] stream_id: A unique stream identifier
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (update_settings, stream_id = None, completion = None))]
    pub fn update_live_stream(
        &self,
        update_settings: PyObject,
        stream_id: Option<&str>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let stream_id_cstr = stream_id
            .map(|id| CString::new(id).expect("invalid stream id string"))
            .or(None);

        let update_settings_map: HashMap<String, DictValue> =
            Python::with_gil(|py| update_settings.extract(py).unwrap());
        let update_settings_string = serde_json::to_string(&update_settings_map).unwrap();
        let update_settings_cstr =
            CString::new(update_settings_string).expect("invalid live stream settings string");

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_update_live_stream(
                call_client.as_mut(),
                request_id,
                update_settings_cstr.as_ptr(),
                stream_id_cstr
                    .as_ref()
                    .map_or(ptr::null_mut(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Adds additional preconfigured endpoints to an existing live stream.
    ///
    /// :param List[str] endpoints: A list of preconfigured live streaming endpoints
    /// :param Optional[str] stream_id: A unique stream identifier
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (endpoints, stream_id = None, completion = None))]
    pub fn add_live_streaming_endpoints(
        &self,
        endpoints: PyObject,
        stream_id: Option<&str>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let list: Vec<DictValue> = Python::with_gil(|py| endpoints.extract(py).unwrap());
        let array = list.iter().map(|v| v.0.clone()).collect();
        let endpoints = LiveStreamEndpoints::PreConfigured {
            pre_configured_endpoints: array,
        };

        let endpoints_string = serde_json::to_string(&endpoints).unwrap();
        let endpoints_cstr =
            CString::new(endpoints_string).expect("invalid live stream endpoints string");

        let stream_id_cstr = stream_id
            .map(|id| CString::new(id).expect("invalid stream id string"))
            .or(None);

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_add_live_streaming_endpoints(
                call_client.as_mut(),
                request_id,
                endpoints_cstr.as_ptr(),
                stream_id_cstr
                    .as_ref()
                    .map_or(ptr::null_mut(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Removes endpoints from an existing live stream.
    ///
    /// :param List[str] endpoints: The list of live streaming endpoints to remove
    /// :param Optional[str] stream_id: A unique stream identifier
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (endpoints, stream_id = None, completion = None))]
    pub fn remove_live_streaming_endpoints(
        &self,
        endpoints: PyObject,
        stream_id: Option<&str>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let list: Vec<DictValue> = Python::with_gil(|py| endpoints.extract(py).unwrap());
        let array = list.iter().map(|v| v.0.clone()).collect();
        let endpoints = LiveStreamEndpoints::PreConfigured {
            pre_configured_endpoints: array,
        };

        let endpoints_string = serde_json::to_string(&endpoints).unwrap();
        let endpoints_cstr =
            CString::new(endpoints_string).expect("invalid live stream endpoints string");

        let stream_id_cstr = stream_id
            .map(|id| CString::new(id).expect("invalid stream id string"))
            .or(None);

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_add_live_streaming_endpoints(
                call_client.as_mut(),
                request_id,
                endpoints_cstr.as_ptr(),
                stream_id_cstr
                    .as_ref()
                    .map_or(ptr::null_mut(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Starts a recording, if recording is enabled for the current room.
    ///
    /// :param Optional[Mapping[str, Any]] streaming_settings: See :ref:`StreamingSettings`
    /// :param Optional[str] stream_id: A unique stream identifier. Multiple recording sessions can be started by specifying a unique ID
    /// :param Optional[bool] force_new: Whether to force a new recording, even if there is already one in progress
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (streaming_settings = None, stream_id = None, force_new = None, completion = None))]
    pub fn start_recording(
        &self,
        streaming_settings: Option<PyObject>,
        stream_id: Option<&str>,
        force_new: Option<bool>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let stream_id = stream_id.map(|id| id.to_string());

        let streaming_settings = streaming_settings.map(|s| {
            let dict: HashMap<String, DictValue> = Python::with_gil(|py| s.extract(py).unwrap());
            dict.iter().map(|(k, v)| (k.clone(), v.0.clone())).collect()
        });

        let properties = StartRecordingProperties {
            instance_id: stream_id,
            streaming_settings,
            force_new,
        };

        let properties_string = serde_json::to_string(&properties).unwrap();
        let properties_cstr =
            Some(CString::new(properties_string).expect("invalid recording properties"));

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_start_recording(
                call_client.as_mut(),
                request_id,
                properties_cstr.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Stops an ongoing recording. If multiple recording instances are running,
    /// each instance must be stopped individually by providing the unique
    /// stream ID.
    ///
    /// :param Optional[str] stream_id: A unique stream identifier
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (stream_id = None, completion = None))]
    pub fn stop_recording(
        &self,
        stream_id: Option<&str>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let stream_id_cstr = stream_id
            .map(|id| CString::new(id).expect("invalid stream id string"))
            .or(None);

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_stop_recording(
                call_client.as_mut(),
                request_id,
                stream_id_cstr
                    .as_ref()
                    .map_or(ptr::null_mut(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Updates an ongoing recording. If multiple recording instances are
    /// running, each instance must be updated individually by providing the
    /// unique stream ID.
    ///
    /// :param Mapping[str, Any] update_settings: See :ref:`StreamingUpdateSettings`
    /// :param Optional[str] stream_id: A unique stream identifier
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (update_settings, stream_id = None, completion = None))]
    pub fn update_recording(
        &self,
        update_settings: PyObject,
        stream_id: Option<&str>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let stream_id_cstr = stream_id
            .map(|id| CString::new(id).expect("invalid stream id string"))
            .or(None);

        let update_settings_map: HashMap<String, DictValue> =
            Python::with_gil(|py| update_settings.extract(py).unwrap());
        let update_settings_string = serde_json::to_string(&update_settings_map).unwrap();
        let update_settings_cstr =
            CString::new(update_settings_string).expect("invalid recording settings string");

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_update_recording(
                call_client.as_mut(),
                request_id,
                update_settings_cstr.as_ptr(),
                stream_id_cstr
                    .as_ref()
                    .map_or(ptr::null_mut(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Starts a transcription service. This can be done by meeting owners or
    /// transcription admins when transcription is enabled in the Daily domain.
    ///
    /// :param Optional[Mapping[str, Any]] settings: See :ref:`TranscriptionSettings`
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (settings = None, completion = None))]
    pub fn start_transcription(
        &self,
        settings: Option<PyObject>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let settings_cstr = settings
            .map(|settings| {
                let settings_map: HashMap<String, DictValue> =
                    Python::with_gil(|py| settings.extract(py).unwrap());
                let settings_string = serde_json::to_string(&settings_map).unwrap();
                CString::new(settings_string).expect("invalid transcription settings string")
            })
            .or(None);

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_start_transcription(
                call_client.as_mut(),
                request_id,
                settings_cstr.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Stops a currently running transcription service. This can be done by
    /// meeting owners or transcription admins when transcription is enabled in
    /// the Daily domain.
    ///
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (completion = None))]
    pub fn stop_transcription(&self, completion: Option<PyObject>) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_stop_transcription(call_client.as_mut(), request_id);
        }

        Ok(())
    }

    /// Updates a transcription service. This allows selecting participants who
    /// should be transcribed (by default all are). This can be done by meeting
    /// owners or transcription admins when transcription is enabled in the
    /// Daily domain.
    ///
    /// :param Optional[List[str]] participants: List of participant IDs who should be transcribed or `None` to transcrible all
    /// :param Optional[str] instance_id: An optional transcription instance ID
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (participants = None, instance_id = None, completion = None))]
    pub fn update_transcription(
        &self,
        participants: Option<PyObject>,
        instance_id: Option<&str>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let participants: Option<Vec<String>> = if let Some(participants) = participants {
            Python::with_gil(|py| participants.extract(py).unwrap())
        } else {
            None
        };
        let participants_string = participants
            .map(|v| serde_json::to_string(&v).unwrap())
            .or(None);
        let participants_cstr = participants_string
            .map(|v| CString::new(v).expect("invalid participant IDs string"))
            .or(None);

        let instance_id_cstr = instance_id
            .map(|p| CString::new(p).expect("invalid instance ID string"))
            .or(None);

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_update_transcription(
                call_client.as_mut(),
                request_id,
                participants_cstr
                    .as_ref()
                    .map_or(ptr::null(), |s| s.as_ptr()),
                instance_id_cstr
                    .as_ref()
                    .map_or(ptr::null(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Starts a dial-out service. This can be done by meeting owners when
    /// dial-out is enabled in the Daily domain.
    ///
    /// :param Optional[Mapping[str, Any]] settings: See :ref:`DialoutSettings`
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (settings = None, completion = None))]
    pub fn start_dialout(
        &self,
        py: Python<'_>,
        settings: Option<PyObject>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let settings_cstr = settings
            .map(|settings| {
                let settings_map: HashMap<String, DictValue> = settings.extract(py).unwrap();
                let settings_string = serde_json::to_string(&settings_map).unwrap();
                CString::new(settings_string).expect("invalid dialout settings string")
            })
            .or(None);

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_start_dialout(
                call_client.as_mut(),
                request_id,
                settings_cstr.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Stops a currently running dial-out service. This can be done by meeting
    /// owners when dial-out is enabled in the Daily domain.
    ///
    /// :param str participant_id: The participant ID of the dial-out session to stop
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (participant_id, completion = None))]
    pub fn stop_dialout(&self, participant_id: &str, completion: Option<PyObject>) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let participant_id_cstr =
            CString::new(participant_id).expect("invalid participant ID string");

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_stop_dialout(
                call_client.as_mut(),
                request_id,
                participant_id_cstr.as_ptr(),
            );
        }

        Ok(())
    }

    /// Sends DTMF tones in an existing dial-out session.
    ///
    /// :param Optional[Mapping[str, Any]] settings: See :ref:`DialoutSendDtmfSettings`
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (settings = None, completion = None))]
    pub fn send_dtmf(
        &self,
        py: Python<'_>,
        settings: Option<PyObject>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let settings_cstr = settings
            .map(|settings| {
                let settings_map: HashMap<String, DictValue> = settings.extract(py).unwrap();
                let settings_string = serde_json::to_string(&settings_map).unwrap();
                CString::new(settings_string).expect("invalid send DTMF settings string")
            })
            .or(None);

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_send_dtmf(
                call_client.as_mut(),
                request_id,
                settings_cstr.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Transfer a SIP dial-in call from one Daily room to another Daily
    /// room. Alternatively, transfer an initiated SIP/PSTN dial-out to another
    /// SIP URI or PSTN number.
    ///
    /// :param Optional[Mapping[str, Any]] settings: See :ref:`SipCallTransferSettings`
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (settings = None, completion = None))]
    pub fn sip_call_transfer(
        &self,
        py: Python<'_>,
        settings: Option<PyObject>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let settings_cstr = settings
            .map(|settings| {
                let settings_map: HashMap<String, DictValue> = settings.extract(py).unwrap();
                let settings_string = serde_json::to_string(&settings_map).unwrap();
                CString::new(settings_string).expect("invalid SIP call transfer settings string")
            })
            .or(None);

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_sip_call_transfer(
                call_client.as_mut(),
                request_id,
                settings_cstr.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Transfer a SIP dial-in call to another SIP endpoint outside Daily.
    ///
    /// :param Optional[Mapping[str, Any]] settings: See :ref:`SipCallTransferSettings`
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (settings = None, completion = None))]
    pub fn sip_refer(
        &self,
        py: Python<'_>,
        settings: Option<PyObject>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let settings_cstr = settings
            .map(|settings| {
                let settings_map: HashMap<String, DictValue> = settings.extract(py).unwrap();
                let settings_string = serde_json::to_string(&settings_map).unwrap();
                CString::new(settings_string).expect("invalid SIP refer settings string")
            })
            .or(None);

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_sip_refer(
                call_client.as_mut(),
                request_id,
                settings_cstr.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Sends a message to other participants, or another specific participant,
    /// during the call.
    ///
    /// :param Any message: The message to send (should be serializable to JSON)
    /// :param Optional[str] participant_id: The participant ID to send the message to. Or `None` to broadcast the message
    /// :param bool serialize_none: Whether `None` should be serialized to `null` (default: true)
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (message, participant_id = None , serialize_none = true, completion = None))]
    pub fn send_app_message(
        &self,
        py: Python<'_>,
        message: PyObject,
        participant_id: Option<&str>,
        serialize_none: bool,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        if message.is_none(py) {
            return Err(exceptions::PyValueError::new_err(format!(
                "invalid app message '{message}'"
            )));
        }

        if let Some(participant_id) = participant_id {
            Uuid::from_str(participant_id).map_err(|_| {
                exceptions::PyValueError::new_err(format!(
                    "invalid participant ID '{participant_id}'"
                ))
            })?;
        }

        let mut message_value: DictValue = message.extract(py)?;
        if !serialize_none {
            message_value.remove_null_fields();
        }
        let message_string = serde_json::to_string(&message_value.0).unwrap();
        let message_cstr = CString::new(message_string).expect("invalid message string");

        let participant_id_cstr = participant_id
            .map(|p| CString::new(p).expect("invalid participant ID string"))
            .or(None);

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_send_app_message(
                call_client.as_mut(),
                request_id,
                message_cstr.as_ptr(),
                participant_id_cstr
                    .as_ref()
                    .map_or(ptr::null(), |s| s.as_ptr()),
            );
        }

        Ok(())
    }

    /// Sends a chat message to Daily's Prebuilt main room.
    ///
    /// :param str message: The chat message to send
    /// :param Optional[str] user_name: The user name that will appear as a sender of the message
    /// :param Optional[func] completion: An optional completion callback with one parameter: (:ref:`CallClientError`)
    #[pyo3(signature = (message, user_name = None, completion = None))]
    pub fn send_prebuilt_chat_message(
        &self,
        message: &str,
        user_name: Option<&str>,
        completion: Option<PyObject>,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let message_cstr = CString::new(message).expect("invalid message string");

        let user_name_cstr = user_name
            .map(|p| CString::new(p).expect("invalid user name string"))
            .or(None);

        let request_id =
            self.maybe_register_completion(completion.map(PyCallClientCompletion::UnaryFn));

        unsafe {
            daily_core_call_client_send_prebuilt_chat_message(
                call_client.as_mut(),
                request_id,
                message_cstr.as_ptr(),
                user_name_cstr.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
                ptr::null(),
            );
        }

        Ok(())
    }

    /// Returns the latest network statistics.
    ///
    /// :return: See :ref:`NetworkStats`
    /// :rtype: Mapping[str, Any]
    pub fn get_network_stats(&self) -> PyResult<PyObject> {
        // If we have already been released throw an exception.
        self.check_released()?;

        Ok(self.inner.network_stats.lock().unwrap().clone())
    }

    /// Registers an audio renderer for the given audio source of the provided
    /// participant.
    ///
    /// :param str participant_id: The ID of the participant to receive audio from
    /// :param func callback: A callback to be called when audio data is available. It receives three arguments: the participant ID, the :class:`AudioData` and the audio source
    /// :param str audio_source: The audio source of the remote participant to receive (e.g. `microphone`, `screenAudio` or a custom track name)
    /// :param str sample_rate: The sample rate the audio should be resampled to if necessary
    /// :param str callback_interval_ms: How often the callback should be called (multiple of 10ms)
    #[pyo3(signature = (participant_id, callback, audio_source = "microphone", sample_rate = 16000, callback_interval_ms = 20))]
    pub fn set_audio_renderer(
        &self,
        participant_id: &str,
        callback: PyObject,
        audio_source: &str,
        sample_rate: u32,
        callback_interval_ms: u32,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let participant_cstr = CString::new(participant_id).expect("invalid participant ID string");
        let audio_source_cstr = CString::new(audio_source).expect("invalid audio source string");

        let request_id = self.maybe_register_completion(None);

        // Use the request_id as our renderer_id (it will be unique anyways) and
        // register the video renderer python callback.
        let renderer_data = AudioRendererData {
            audio_source: audio_source.to_string(),
            callback,
            audio_buffer: Vec::new(),
            callback_interval_ms,
            callback_count: 0,
        };
        self.inner
            .audio_renderers
            .lock()
            .unwrap()
            .insert(request_id, renderer_data);

        unsafe {
            daily_core_call_client_set_participant_audio_renderer(
                call_client.as_mut(),
                request_id,
                request_id,
                participant_cstr.as_ptr(),
                audio_source_cstr.as_ptr(),
                sample_rate,
            );
        }

        Ok(())
    }

    /// Registers a video renderer for the given video source of the provided
    /// participant. The color format of the received frames can be chosen.
    ///
    /// :param str participant_id: The ID of the participant to receive video from
    /// :param func callback: A callback to be called on every received frame. It receives three arguments: the participant ID, a :class:`VideoFrame` and the video source
    /// :param str video_source: The video source of the remote participant to receive (e.g. `camera`, `screenVideo` or a custom track name)
    /// :param str color_format: The color format that frames should be received. See :ref:`ColorFormat`
    #[pyo3(signature = (participant_id, callback, video_source = "camera", color_format = "RGBA"))]
    pub fn set_video_renderer(
        &self,
        participant_id: &str,
        callback: PyObject,
        video_source: &str,
        color_format: &str,
    ) -> PyResult<()> {
        // If we have already been released throw an exception.
        let mut call_client = self.check_released()?;

        let participant_cstr = CString::new(participant_id).expect("invalid participant ID string");
        let video_source_cstr = CString::new(video_source).expect("invalid video source string");
        let color_format_cstr = CString::new(color_format).expect("invalid color format string");

        if ColorFormat::from_str(color_format).is_err() {
            return Err(exceptions::PyValueError::new_err(format!(
                "invalid color format '{color_format}'"
            )));
        }

        let request_id = self.maybe_register_completion(None);

        // Use the request_id as our renderer_id (it will be unique anyways) and
        // register the video renderer python callback.
        let renderer_data = VideoRendererData {
            video_source: video_source.to_string(),
            callback,
        };
        self.inner
            .video_renderers
            .lock()
            .unwrap()
            .insert(request_id, renderer_data);

        unsafe {
            daily_core_call_client_set_participant_video_renderer(
                call_client.as_mut(),
                request_id,
                request_id,
                participant_cstr.as_ptr(),
                video_source_cstr.as_ptr(),
                color_format_cstr.as_ptr(),
            );
        }

        Ok(())
    }
}

impl Drop for PyCallClient {
    // GIL acquired
    fn drop(&mut self) {
        // We know the GIL is acquired because it is acquired before
        // dropping a pyclass object.
        let py = unsafe { Python::assume_gil_acquired() };

        let _ = self.release(py);
    }
}

unsafe fn get_active_speaker(call_client: &mut CallClient) -> PyResult<PyObject> {
    let active_speaker_ptr = daily_core_call_client_active_speaker(call_client);
    let active_speaker_string = CStr::from_ptr(active_speaker_ptr)
        .to_string_lossy()
        .into_owned();

    let active_speaker: Option<HashMap<String, DictValue>> =
        serde_json::from_str(active_speaker_string.as_str()).unwrap();

    Python::with_gil(|py| Ok(active_speaker.to_object(py)))
}

unsafe fn get_inputs(call_client: &mut CallClient) -> PyResult<PyObject> {
    let inputs_ptr = daily_core_call_client_inputs(call_client);
    let inputs_string = CStr::from_ptr(inputs_ptr).to_string_lossy().into_owned();

    let inputs: HashMap<String, DictValue> = serde_json::from_str(inputs_string.as_str()).unwrap();

    Python::with_gil(|py| Ok(inputs.to_object(py)))
}

unsafe fn get_participant_counts(call_client: &mut CallClient) -> PyResult<PyObject> {
    let participant_counts_ptr = daily_core_call_client_participant_counts(call_client);
    let participant_counts_string = CStr::from_ptr(participant_counts_ptr)
        .to_string_lossy()
        .into_owned();

    let participant_counts: HashMap<String, DictValue> =
        serde_json::from_str(participant_counts_string.as_str()).unwrap();

    Python::with_gil(|py| Ok(participant_counts.to_object(py)))
}

unsafe fn get_publishing(call_client: &mut CallClient) -> PyResult<PyObject> {
    let publishing_ptr = daily_core_call_client_publishing(call_client);
    let publishing_string = CStr::from_ptr(publishing_ptr)
        .to_string_lossy()
        .into_owned();

    let publishing: HashMap<String, DictValue> =
        serde_json::from_str(publishing_string.as_str()).unwrap();

    Python::with_gil(|py| Ok(publishing.to_object(py)))
}

unsafe fn get_subscriptions(call_client: &mut CallClient) -> PyResult<PyObject> {
    let subscriptions_ptr = daily_core_call_client_subscriptions(call_client);
    let subscriptions_string = CStr::from_ptr(subscriptions_ptr)
        .to_string_lossy()
        .into_owned();

    let subscriptions: HashMap<String, DictValue> =
        serde_json::from_str(subscriptions_string.as_str()).unwrap();

    Python::with_gil(|py| Ok(subscriptions.to_object(py)))
}

unsafe fn get_subscription_profiles(call_client: &mut CallClient) -> PyResult<PyObject> {
    let profiles_ptr = daily_core_call_client_subscription_profiles(call_client);
    let profiles_string = CStr::from_ptr(profiles_ptr).to_string_lossy().into_owned();

    let profiles: HashMap<String, DictValue> =
        serde_json::from_str(profiles_string.as_str()).unwrap();

    Python::with_gil(|py| Ok(profiles.to_object(py)))
}

unsafe fn get_network_stats(call_client: &mut CallClient) -> PyResult<PyObject> {
    let stats_ptr = daily_core_call_client_get_network_stats(call_client);
    let stats_string = CStr::from_ptr(stats_ptr).to_string_lossy().into_owned();

    let stats: HashMap<String, DictValue> = serde_json::from_str(stats_string.as_str()).unwrap();

    Python::with_gil(|py| Ok(stats.to_object(py)))
}
