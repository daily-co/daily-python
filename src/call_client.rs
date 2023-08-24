use std::boxed::Box;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr;

use crate::DictValue;
use crate::GLOBAL_CONTEXT;

use daily_core::prelude::{
    daily_core_call_client_create, daily_core_call_client_inputs, daily_core_call_client_join,
    daily_core_call_client_leave, daily_core_call_client_set_participant_video_renderer,
    daily_core_call_client_subscription_profiles, daily_core_call_client_subscriptions,
    daily_core_call_client_update_inputs, daily_core_call_client_update_subscription_profiles,
    daily_core_call_client_update_subscriptions, CallClient, NativeCallClientDelegatePtr,
    NativeCallClientVideoRenderer, NativeCallClientVideoRendererFns, NativeVideoFrame,
};

use pyo3::ffi::Py_IncRef;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};

#[pyclass(name = "CallClientCallbackContext", module = "daily")]
pub struct PyCallClientCallbackContext {
    pub callback: PyObject,
}

#[pyclass(name = "CallClient", module = "daily")]
pub struct PyCallClient {
    call_client: Box<CallClient>,
}

#[pymethods]
impl PyCallClient {
    #[new]
    pub fn new() -> Self {
        unsafe {
            let call_client = daily_core_call_client_create();

            Self {
                call_client: Box::from_raw(call_client),
            }
        }
    }

    #[pyo3(signature = (meeting_url, py_meeting_token = None, py_client_settings = None))]
    pub fn join(
        &mut self,
        meeting_url: &str,
        py_meeting_token: Option<PyObject>,
        py_client_settings: Option<PyObject>,
    ) {
        unsafe {
            // Meeting URL
            let meeting_url_string = CString::new(meeting_url)
                .expect("invalid meeting URL string")
                .into_raw();

            // Meeting token
            let meeting_token: String = if let Some(py_meeting_token) = py_meeting_token {
                Python::with_gil(|py| py_meeting_token.extract(py).unwrap())
            } else {
                "".to_string()
            };
            let meeting_token_string = if meeting_token.is_empty() {
                ptr::null_mut()
            } else {
                CString::new(meeting_token)
                    .expect("invalid meeting token string")
                    .into_raw()
            };

            // Client settings
            let client_settings: String = if let Some(py_client_settings) = py_client_settings {
                Python::with_gil(|py| {
                    let client_settings: HashMap<String, DictValue> =
                        py_client_settings.extract(py).unwrap();
                    serde_json::to_string(&client_settings).unwrap()
                })
            } else {
                "".to_string()
            };
            let client_settings_string = if client_settings.is_empty() {
                ptr::null_mut()
            } else {
                CString::new(client_settings)
                    .expect("invalid client settings string")
                    .into_raw()
            };

            // Join
            daily_core_call_client_join(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                meeting_url_string,
                meeting_token_string,
                client_settings_string,
            );

            let _ = CString::from_raw(meeting_url_string);
            if !meeting_token_string.is_null() {
                let _ = CString::from_raw(meeting_token_string);
            }
            if !client_settings_string.is_null() {
                let _ = CString::from_raw(client_settings_string);
            }
        }
    }

    pub fn leave(&mut self) {
        unsafe {
            daily_core_call_client_leave(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
            );
        }
    }

    pub fn inputs(&mut self) -> PyObject {
        unsafe {
            let inputs_ptr = daily_core_call_client_inputs(self.call_client.as_mut());
            let inputs_string = CStr::from_ptr(inputs_ptr).to_string_lossy().into_owned();

            let inputs: HashMap<String, DictValue> =
                serde_json::from_str(inputs_string.as_str()).unwrap();

            Python::with_gil(|py| inputs.to_object(py))
        }
    }

    pub fn update_inputs(&mut self, py_input_settings: PyObject) {
        unsafe {
            let input_settings: HashMap<String, DictValue> =
                Python::with_gil(|py| py_input_settings.extract(py).unwrap());

            let input_settings_string = serde_json::to_string(&input_settings).unwrap();

            daily_core_call_client_update_inputs(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                CString::new(input_settings_string)
                    .expect("invalid input settings string")
                    .into_raw(),
            );
        }
    }

    pub fn subscriptions(&mut self) -> PyObject {
        unsafe {
            let subscriptions_ptr = daily_core_call_client_subscriptions(self.call_client.as_mut());
            let subscriptions_string = CStr::from_ptr(subscriptions_ptr)
                .to_string_lossy()
                .into_owned();

            let subscriptions: HashMap<String, DictValue> =
                serde_json::from_str(subscriptions_string.as_str()).unwrap();

            Python::with_gil(|py| subscriptions.to_object(py))
        }
    }

    pub fn update_subscriptions(
        &mut self,
        py_participant_settings: PyObject,
        py_profile_settings: PyObject,
    ) {
        unsafe {
            let participant_settings: HashMap<String, DictValue> =
                Python::with_gil(|py| py_participant_settings.extract(py).unwrap());
            let profile_settings: HashMap<String, DictValue> =
                Python::with_gil(|py| py_profile_settings.extract(py).unwrap());

            let participant_settings_string = serde_json::to_string(&participant_settings).unwrap();
            let profile_settings_string = serde_json::to_string(&profile_settings).unwrap();

            daily_core_call_client_update_subscriptions(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                CString::new(participant_settings_string)
                    .expect("invalid participant settings string")
                    .into_raw(),
                CString::new(profile_settings_string)
                    .expect("invalid profile settings string")
                    .into_raw(),
            );
        }
    }

    pub fn subscription_profiles(&mut self) -> PyObject {
        unsafe {
            let profiles_ptr =
                daily_core_call_client_subscription_profiles(self.call_client.as_mut());
            let profiles_string = CStr::from_ptr(profiles_ptr).to_string_lossy().into_owned();

            let profiles: HashMap<String, DictValue> =
                serde_json::from_str(profiles_string.as_str()).unwrap();

            Python::with_gil(|py| profiles.to_object(py))
        }
    }

    pub fn update_subscription_profiles(&mut self, py_profile_settings: PyObject) {
        unsafe {
            let profile_settings: HashMap<String, DictValue> =
                Python::with_gil(|py| py_profile_settings.extract(py).unwrap());

            let profile_settings_string = serde_json::to_string(&profile_settings).unwrap();

            daily_core_call_client_update_subscription_profiles(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                CString::new(profile_settings_string)
                    .expect("invalid profile settings string")
                    .into_raw(),
            );
        }
    }

    #[pyo3(signature = (participant_id, callback, video_source = "camera", color_format = "RGBA32"))]
    pub fn set_video_renderer(
        &mut self,
        participant_id: &str,
        callback: PyObject,
        video_source: &str,
        color_format: &str,
    ) {
        unsafe {
            let participant_ptr = CString::new(participant_id)
                .expect("invalid participant ID string")
                .into_raw();

            let video_source_ptr = CString::new(video_source)
                .expect("invalid video source string")
                .into_raw();

            let color_format_ptr = CString::new(color_format)
                .expect("invalid color format string")
                .into_raw();

            let callback_ctx: PyObject = Python::with_gil(|py| {
                Py::new(py, PyCallClientCallbackContext { callback })
                    .unwrap()
                    .into_py(py)
            });

            let video_renderer = NativeCallClientVideoRenderer {
                ptr: NativeCallClientDelegatePtr(callback_ctx.into_ptr() as *mut libc::c_void),
                fns: NativeCallClientVideoRendererFns { on_video_frame },
            };

            daily_core_call_client_set_participant_video_renderer(
                self.call_client.as_mut(),
                participant_ptr,
                video_source_ptr,
                color_format_ptr,
                video_renderer,
            );

            let _ = CString::from_raw(participant_ptr);
            let _ = CString::from_raw(video_source_ptr);
            let _ = CString::from_raw(color_format_ptr);
        }
    }
}

#[pyclass(name = "VideoFrame", module = "daily", get_all, unsendable)]
struct PyVideoFrame {
    pub buffer: PyObject,
    pub width: i32,
    pub height: i32,
    pub timestamp_us: i64,
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

        let video_frame = PyVideoFrame {
            buffer: PyBytes::from_ptr(py, (*frame).buffer, (*frame).buffer_size).into_py(py),
            width: (*frame).width,
            height: (*frame).height,
            timestamp_us: (*frame).timestamp_us,
        };

        let args = PyTuple::new(py, &[peer_id.into_py(py), video_frame.into_py(py)]);

        let _ = callback_ctx.callback.call1(py, args);
    });
}

impl Drop for PyCallClient {
    fn drop(&mut self) {
        self.leave();
    }
}
