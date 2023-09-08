use std::boxed::Box;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr;

use crate::dict::DictValue;
use crate::event::{args_from_event, method_name_from_event, Event};
use crate::PyVideoFrame;
use crate::GLOBAL_CONTEXT;

use daily_core::prelude::{
    daily_core_call_client_create, daily_core_call_client_inputs, daily_core_call_client_join,
    daily_core_call_client_leave, daily_core_call_client_participant_counts,
    daily_core_call_client_participants, daily_core_call_client_publishing,
    daily_core_call_client_set_delegate, daily_core_call_client_set_participant_video_renderer,
    daily_core_call_client_set_user_name, daily_core_call_client_subscription_profiles,
    daily_core_call_client_subscriptions, daily_core_call_client_update_inputs,
    daily_core_call_client_update_permissions, daily_core_call_client_update_publishing,
    daily_core_call_client_update_remote_participants,
    daily_core_call_client_update_subscription_profiles,
    daily_core_call_client_update_subscriptions, CallClient, NativeCallClientDelegate,
    NativeCallClientDelegateFns, NativeCallClientDelegatePtr, NativeCallClientVideoRenderer,
    NativeCallClientVideoRendererFns, NativeVideoFrame,
};

use pyo3::exceptions;
use pyo3::ffi::Py_IncRef;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};

#[pyclass(name = "CallClientCallbackContext", module = "daily")]
pub struct PyCallClientCallbackContext {
    pub callback: PyObject,
}

/// This class represents a call client. A call client is a participant of a
/// Daily meeting and it can receive audio and video from other participants in
/// the meeting as well as send audio and video. Multiple instances of call
/// clients can be created in the same application.
///
/// :param class event_handler: A subclass of :class:`daily.EventHandler`
#[pyclass(name = "CallClient", module = "daily")]
pub struct PyCallClient {
    call_client: Box<CallClient>,
}

#[pymethods]
impl PyCallClient {
    /// Create a new call client. The new call client can receive meeting events
    /// through an event handler.
    #[new]
    pub fn new(event_handler: Option<PyObject>) -> PyResult<Self> {
        unsafe {
            let call_client = daily_core_call_client_create();
            if !call_client.is_null() {
                if let Some(event_handler) = event_handler {
                    let callback_ctx: PyObject = Python::with_gil(|py| {
                        Py::new(
                            py,
                            PyCallClientCallbackContext {
                                callback: event_handler,
                            },
                        )
                        .unwrap()
                        .into_py(py)
                    });

                    let client_delegate = NativeCallClientDelegate::new(
                        NativeCallClientDelegatePtr::new(
                            callback_ctx.into_ptr() as *mut libc::c_void
                        ),
                        NativeCallClientDelegateFns::new(on_event),
                    );

                    daily_core_call_client_set_delegate(&mut (*call_client), client_delegate);
                }

                Ok(Self {
                    call_client: Box::from_raw(call_client),
                })
            } else {
                Err(exceptions::PyRuntimeError::new_err(
                    "unable to create a CallClient() object",
                ))
            }
        }
    }

    /// Join a meeting given the `meeting_url` and the optional `meeting_token`
    /// and `client_settings`. The client settings specifie inputs updates or
    /// publising settings.
    ///
    /// :param str meeting_url: The URL of the Daily meeting to join
    /// :param str meeting_token: Meeting token if needed. This is needed if the client is an owner of the meeting
    /// :param dict client_settings: See :ref:`ClientSettings`
    #[pyo3(signature = (meeting_url, meeting_token = None, client_settings = None))]
    pub fn join(
        &mut self,
        meeting_url: &str,
        meeting_token: Option<PyObject>,
        client_settings: Option<PyObject>,
    ) {
        // Meeting URL
        let meeting_url_cstr = CString::new(meeting_url).expect("invalid meeting URL string");

        // Meeting token
        let meeting_token_string: String = if let Some(meeting_token) = meeting_token {
            Python::with_gil(|py| meeting_token.extract(py).unwrap())
        } else {
            "".to_string()
        };
        let meeting_token_cstr =
            CString::new(meeting_token_string).expect("invalid meeting token string");

        // Client settings
        let client_settings_string: String = if let Some(client_settings) = client_settings {
            Python::with_gil(|py| {
                let client_settings: HashMap<String, DictValue> =
                    client_settings.extract(py).unwrap();
                serde_json::to_string(&client_settings).unwrap()
            })
        } else {
            "".to_string()
        };
        let client_settings_cstr =
            CString::new(client_settings_string).expect("invalid client settings string");

        unsafe {
            daily_core_call_client_join(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                meeting_url_cstr.as_ptr(),
                if meeting_token_cstr.is_empty() {
                    ptr::null_mut()
                } else {
                    meeting_token_cstr.as_ptr()
                },
                if client_settings_cstr.is_empty() {
                    ptr::null_mut()
                } else {
                    client_settings_cstr.as_ptr()
                },
            );
        }
    }

    /// Leave a previously joined meeting.
    pub fn leave(&mut self) {
        unsafe {
            daily_core_call_client_leave(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
            );
        }
    }

    /// Sets this client's user name. The user name is what other participants
    /// might be able to see as a description of this client.
    ///
    /// :param str user_name: This client's user name
    pub fn set_user_name(&mut self, user_name: &str) {
        let user_name_cstr = CString::new(user_name).expect("invalid user name string");

        unsafe {
            daily_core_call_client_set_user_name(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                user_name_cstr.as_ptr(),
            );
        }
    }

    /// Returns the current participants in the meeting.
    ///
    /// :return: See :ref:`CallParticipants`
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
    pub fn update_remote_participants(&mut self, remote_participants: PyObject) {
        let remote_participants_map: HashMap<String, DictValue> =
            Python::with_gil(|py| remote_participants.extract(py).unwrap());

        let remote_participants_string = serde_json::to_string(&remote_participants_map).unwrap();

        let remote_participants_cstr =
            CString::new(remote_participants_string).expect("invalid remote participants string");

        unsafe {
            daily_core_call_client_update_remote_participants(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
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
    pub fn update_inputs(&mut self, input_settings: PyObject) {
        let input_settings_map: HashMap<String, DictValue> =
            Python::with_gil(|py| input_settings.extract(py).unwrap());

        let input_settings_string = serde_json::to_string(&input_settings_map).unwrap();

        let input_settings_cstr =
            CString::new(input_settings_string).expect("invalid input settings string");

        unsafe {
            daily_core_call_client_update_inputs(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
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
    pub fn update_publishing(&mut self, publishing_settings: PyObject) {
        let publishing_settings_map: HashMap<String, DictValue> =
            Python::with_gil(|py| publishing_settings.extract(py).unwrap());

        let publishing_settings_string = serde_json::to_string(&publishing_settings_map).unwrap();

        let publishing_settings_cstr =
            CString::new(publishing_settings_string).expect("invalid publishing settings string");

        unsafe {
            daily_core_call_client_update_publishing(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
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
    #[pyo3(signature = (participant_settings = None, profile_settings = None))]
    pub fn update_subscriptions(
        &mut self,
        participant_settings: Option<PyObject>,
        profile_settings: Option<PyObject>,
    ) {
        // Participant subscription settings
        let participant_settings_string = if let Some(participant_settings) = participant_settings {
            let participant_settings_map: HashMap<String, DictValue> =
                Python::with_gil(|py| participant_settings.extract(py).unwrap());

            serde_json::to_string(&participant_settings_map).unwrap()
        } else {
            "".to_string()
        };

        let participant_settings_cstr =
            CString::new(participant_settings_string).expect("invalid participant settings string");

        // Profile settings
        let profile_settings_string = if let Some(profile_settings) = profile_settings {
            let profile_settings_map: HashMap<String, DictValue> =
                Python::with_gil(|py| profile_settings.extract(py).unwrap());

            serde_json::to_string(&profile_settings_map).unwrap()
        } else {
            "".to_string()
        };

        let profile_settings_cstr =
            CString::new(profile_settings_string).expect("invalid profile settings string");

        unsafe {
            daily_core_call_client_update_subscriptions(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                if participant_settings_cstr.is_empty() {
                    ptr::null()
                } else {
                    participant_settings_cstr.as_ptr()
                },
                if profile_settings_cstr.is_empty() {
                    ptr::null()
                } else {
                    profile_settings_cstr.as_ptr()
                },
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
    pub fn update_subscription_profiles(&mut self, profile_settings: PyObject) {
        let profile_settings_map: HashMap<String, DictValue> =
            Python::with_gil(|py| profile_settings.extract(py).unwrap());

        let profile_settings_string = serde_json::to_string(&profile_settings_map).unwrap();
        let profile_settings_cstr =
            CString::new(profile_settings_string).expect("invalid profile settings string");

        unsafe {
            daily_core_call_client_update_subscription_profiles(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                profile_settings_cstr.as_ptr(),
            );
        }
    }

    /// Updates the client permissions. This will only update permissions for
    /// this client and is only allowed if this client is the owner of the
    /// meeting.
    ///
    /// :param dict permissions: See :ref:`Permissions`
    pub fn update_permissions(&mut self, permissions: PyObject) {
        let permissions_map: HashMap<String, DictValue> =
            Python::with_gil(|py| permissions.extract(py).unwrap());

        let permissions_string = serde_json::to_string(&permissions_map).unwrap();
        let permissions_cstr =
            CString::new(permissions_string).expect("invalid permissions string");

        unsafe {
            daily_core_call_client_update_permissions(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                permissions_cstr.as_ptr(),
            );
        }
    }

    /// Registers a video renderer for the given video source of the provided
    /// participant. The color format of the received frames can be chosen.
    ///
    /// :param str participant_id: The ID of the participant to receive video from
    /// :param function callback: A function or class method to be called on every received frame
    /// :param str video_source: The video source of the remote participant to receive (e.g. `camera`, `screenVideo` or a custom track name)
    /// :param str color_format: The color format that frames should be received. See :ref:`ColorFormat`
    #[pyo3(signature = (participant_id, callback, video_source = "camera", color_format = "RGBA32"))]
    pub fn set_video_renderer(
        &mut self,
        participant_id: &str,
        callback: PyObject,
        video_source: &str,
        color_format: &str,
    ) {
        let participant_cstr = CString::new(participant_id).expect("invalid participant ID string");
        let video_source_cstr = CString::new(video_source).expect("invalid video source string");
        let color_format_cstr = CString::new(color_format).expect("invalid color format string");

        let callback_ctx: PyObject = Python::with_gil(|py| {
            Py::new(py, PyCallClientCallbackContext { callback })
                .unwrap()
                .into_py(py)
        });

        let video_renderer = NativeCallClientVideoRenderer::new(
            NativeCallClientDelegatePtr::new(callback_ctx.into_ptr() as *mut libc::c_void),
            NativeCallClientVideoRendererFns::new(on_video_frame),
        );

        unsafe {
            daily_core_call_client_set_participant_video_renderer(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                participant_cstr.as_ptr(),
                video_source_cstr.as_ptr(),
                color_format_cstr.as_ptr(),
                video_renderer,
            );
        }
    }
}

impl Drop for PyCallClient {
    fn drop(&mut self) {
        self.leave();
    }
}

unsafe extern "C" fn on_event(
    delegate: *mut libc::c_void,
    event_json: *const libc::c_char,
    _json_len: isize,
) {
    Python::with_gil(|py| {
        let py_callback_ctx_ptr = delegate as *mut pyo3::ffi::PyObject;

        // PyObject below will decrease the reference counter and we need to
        // always keep a reference.
        Py_IncRef(py_callback_ctx_ptr);

        let py_callback_ctx = PyObject::from_owned_ptr(py, py_callback_ctx_ptr);

        let callback_ctx: PyRefMut<'_, PyCallClientCallbackContext> =
            py_callback_ctx.extract(py).unwrap();

        let event_string = CStr::from_ptr(event_json).to_string_lossy().into_owned();

        println!("EVENT: {event_string}");

        let event = serde_json::from_str::<Event>(event_string.as_str()).unwrap();

        if let Some(method_name) = method_name_from_event(&event) {
            if let Some(args) = args_from_event(&event) {
                let py_args = PyTuple::new(py, args);

                if let Err(error) = callback_ctx.callback.call_method1(py, method_name, py_args) {
                    error.write_unraisable(py, None);
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
        let py_callback_ctx_ptr = delegate as *mut pyo3::ffi::PyObject;

        // PyObject below will decrease the reference counter and we need to
        // always keep a reference.
        Py_IncRef(py_callback_ctx_ptr);

        let py_callback_ctx = PyObject::from_owned_ptr(py, py_callback_ctx_ptr);

        let callback_ctx: PyRefMut<'_, PyCallClientCallbackContext> =
            py_callback_ctx.extract(py).unwrap();

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

        if let Err(error) = callback_ctx.callback.call1(py, args) {
            error.write_unraisable(py, None);
        }
    });
}
