use std::boxed::Box;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr;

use crate::DictValue;
use crate::GLOBAL_CONTEXT;

use daily_core::prelude::{
    daily_core_call_client_create, daily_core_call_client_join, daily_core_call_client_leave,
    daily_core_call_client_set_participant_camera_renderer,
    daily_core_call_client_subscription_profiles, daily_core_call_client_subscriptions,
    daily_core_call_client_update_subscription_profiles,
    daily_core_call_client_update_subscriptions, CallClient, NativeCallClientDelegatePtr,
    NativeCallClientVideoRenderer, NativeCallClientVideoRendererFns, NativeVideoFrame,
};

use pyo3::ffi::Py_IncRef;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};

#[pyclass(name = "CallClient", module = "daily")]
pub(crate) struct PyCallClient {
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

    #[pyo3(signature = (meeting_url, meeting_token = "", client_settings = ""))]
    pub fn join(&mut self, meeting_url: &str, meeting_token: &str, client_settings: &str) {
        unsafe {
            let url = CString::new(meeting_url)
                .expect("Invalid meeting URL string")
                .into_raw();
            let token = if meeting_token.is_empty() {
                ptr::null_mut()
            } else {
                CString::new(meeting_token)
                    .expect("Invalid meeting token string")
                    .into_raw()
            };
            let settings = if client_settings.is_empty() {
                ptr::null_mut()
            } else {
                CString::new(client_settings)
                    .expect("Invalid client settings string")
                    .into_raw()
            };

            daily_core_call_client_join(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                url,
                token,
                settings,
            );

            let _ = CString::from_raw(url);
            if !token.is_null() {
                let _ = CString::from_raw(token);
            }
            if !settings.is_null() {
                let _ = CString::from_raw(settings);
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
            Python::with_gil(|py| {
                let participant_settings: HashMap<String, DictValue> =
                    py_participant_settings.extract(py).unwrap();
                let profile_settings: HashMap<String, DictValue> =
                    py_profile_settings.extract(py).unwrap();

                let participant_settings_string =
                    serde_json::to_string(&participant_settings).unwrap();
                let profile_settings_string = serde_json::to_string(&profile_settings).unwrap();

                daily_core_call_client_update_subscriptions(
                    self.call_client.as_mut(),
                    GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                    CString::new(participant_settings_string)
                        .expect("Invalid participant settings string")
                        .into_raw(),
                    CString::new(profile_settings_string)
                        .expect("Invalid profile settings string")
                        .into_raw(),
                );
            });
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
            Python::with_gil(|py| {
                let profile_settings: HashMap<String, DictValue> =
                    py_profile_settings.extract(py).unwrap();

                let profile_settings_string = serde_json::to_string(&profile_settings).unwrap();

                daily_core_call_client_update_subscription_profiles(
                    self.call_client.as_mut(),
                    GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                    CString::new(profile_settings_string)
                        .expect("Invalid profile settings string")
                        .into_raw(),
                );
            });
        }
    }

    pub fn set_camera_renderer(&mut self, participant: &str, callback: PyObject, ctx: PyObject) {
        unsafe {
            let participant_ptr = CString::new(participant)
                .expect("Invalid participant ID string")
                .into_raw();

            Python::with_gil(|py| {
                let callback_ctx: PyObject = Py::new(py, PyCallbackContext { callback, ctx })
                    .unwrap()
                    .into_py(py);

                let video_renderer = NativeCallClientVideoRenderer {
                    ptr: NativeCallClientDelegatePtr(callback_ctx.into_ptr() as *mut libc::c_void),
                    fns: NativeCallClientVideoRendererFns { on_video_frame },
                };

                daily_core_call_client_set_participant_camera_renderer(
                    self.call_client.as_mut(),
                    participant_ptr,
                    video_renderer,
                );

                let _ = CString::from_raw(participant_ptr);
            })
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

#[pyclass(name = "CallbackContext", module = "daily")]
struct PyCallbackContext {
    pub callback: PyObject,
    pub ctx: PyObject,
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

        let callback_ctx: PyRefMut<'_, PyCallbackContext> = py_callback_ctx.extract(py).unwrap();

        let peer_id = CStr::from_ptr(peer_id).to_string_lossy().into_owned();

        let video_frame = PyVideoFrame {
            buffer: PyBytes::from_ptr(py, (*frame).buffer, (*frame).buffer_size).into_py(py),
            width: (*frame).width,
            height: (*frame).height,
            timestamp_us: (*frame).timestamp_us,
        };

        let args = PyTuple::new(
            py,
            &[
                callback_ctx.ctx.clone_ref(py),
                peer_id.into_py(py),
                video_frame.into_py(py),
            ],
        );

        let _ = callback_ctx.callback.call1(py, args);
    });
}
