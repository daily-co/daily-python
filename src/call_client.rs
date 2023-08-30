use std::boxed::Box;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr;

use crate::DictValue;
use crate::PyVideoFrame;
use crate::GLOBAL_CONTEXT;

use daily_core::prelude::{
    daily_core_call_client_create, daily_core_call_client_inputs, daily_core_call_client_join,
    daily_core_call_client_leave, daily_core_call_client_participant_counts,
    daily_core_call_client_participants, daily_core_call_client_set_participant_video_renderer,
    daily_core_call_client_set_user_name, daily_core_call_client_subscription_profiles,
    daily_core_call_client_subscriptions, daily_core_call_client_update_inputs,
    daily_core_call_client_update_permissions, daily_core_call_client_update_remote_participants,
    daily_core_call_client_update_subscription_profiles,
    daily_core_call_client_update_subscriptions, CallClient, NativeCallClientDelegatePtr,
    NativeCallClientVideoRenderer, NativeCallClientVideoRendererFns, NativeVideoFrame,
};

use pyo3::exceptions;
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
    pub fn new() -> PyResult<Self> {
        unsafe {
            let call_client = daily_core_call_client_create();
            if !call_client.is_null() {
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

    #[pyo3(signature = (meeting_url, py_meeting_token = None, py_client_settings = None))]
    pub fn join(
        &mut self,
        meeting_url: &str,
        py_meeting_token: Option<PyObject>,
        py_client_settings: Option<PyObject>,
    ) {
        // Meeting URL
        let meeting_url_ptr = CString::new(meeting_url)
            .expect("invalid meeting URL string")
            .into_raw();

        // Meeting token
        let meeting_token: String = if let Some(py_meeting_token) = py_meeting_token {
            Python::with_gil(|py| py_meeting_token.extract(py).unwrap())
        } else {
            "".to_string()
        };
        let meeting_token_ptr = if meeting_token.is_empty() {
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
        let client_settings_ptr = if client_settings.is_empty() {
            ptr::null_mut()
        } else {
            CString::new(client_settings)
                .expect("invalid client settings string")
                .into_raw()
        };

        unsafe {
            daily_core_call_client_join(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                meeting_url_ptr,
                meeting_token_ptr,
                client_settings_ptr,
            );

            let _ = CString::from_raw(meeting_url_ptr);
            if !meeting_token_ptr.is_null() {
                let _ = CString::from_raw(meeting_token_ptr);
            }
            if !client_settings_ptr.is_null() {
                let _ = CString::from_raw(client_settings_ptr);
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

    pub fn set_user_name(&mut self, user_name: &str) {
        unsafe {
            let user_name_ptr = CString::new(user_name)
                .expect("invalid user name string")
                .into_raw();

            daily_core_call_client_set_user_name(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                user_name_ptr,
            );

            let _ = CString::from_raw(user_name_ptr);
        }
    }

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

    pub fn update_remote_participants(&mut self, py_remote_participants: PyObject) {
        let remote_participants: HashMap<String, DictValue> =
            Python::with_gil(|py| py_remote_participants.extract(py).unwrap());

        let remote_participants_string = serde_json::to_string(&remote_participants).unwrap();

        let remote_participants_ptr = CString::new(remote_participants_string)
            .expect("invalid remote participants string")
            .into_raw();

        unsafe {
            daily_core_call_client_update_remote_participants(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                remote_participants_ptr,
            );

            let _ = CString::from_raw(remote_participants_ptr);
        }
    }

    pub fn inputs(&mut self) -> PyResult<PyObject> {
        unsafe {
            let inputs_ptr = daily_core_call_client_inputs(self.call_client.as_mut());
            let inputs_string = CStr::from_ptr(inputs_ptr).to_string_lossy().into_owned();

            let inputs: HashMap<String, DictValue> =
                serde_json::from_str(inputs_string.as_str()).unwrap();

            Ok(Python::with_gil(|py| inputs.to_object(py)))
        }
    }

    pub fn update_inputs(&mut self, py_input_settings: PyObject) {
        let input_settings: HashMap<String, DictValue> =
            Python::with_gil(|py| py_input_settings.extract(py).unwrap());

        let input_settings_string = serde_json::to_string(&input_settings).unwrap();

        let input_settings_ptr = CString::new(input_settings_string)
            .expect("invalid input settings string")
            .into_raw();

        unsafe {
            daily_core_call_client_update_inputs(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                input_settings_ptr,
            );

            let _ = CString::from_raw(input_settings_ptr);
        }
    }

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

    #[pyo3(signature = (py_participant_settings = None, py_profile_settings = None))]
    pub fn update_subscriptions(
        &mut self,
        py_participant_settings: Option<PyObject>,
        py_profile_settings: Option<PyObject>,
    ) {
        let participant_settings_ptr = if let Some(py_participant_settings) =
            py_participant_settings
        {
            let participant_settings: HashMap<String, DictValue> =
                Python::with_gil(|py| py_participant_settings.extract(py).unwrap());

            let participant_settings_string = serde_json::to_string(&participant_settings).unwrap();

            CString::new(participant_settings_string)
                .expect("invalid participant settings string")
                .into_raw()
        } else {
            ptr::null_mut()
        };

        let profile_settings_ptr = if let Some(py_profile_settings) = py_profile_settings {
            let profile_settings: HashMap<String, DictValue> =
                Python::with_gil(|py| py_profile_settings.extract(py).unwrap());

            let profile_settings_string = serde_json::to_string(&profile_settings).unwrap();

            CString::new(profile_settings_string)
                .expect("invalid profile settings string")
                .into_raw()
        } else {
            ptr::null_mut()
        };

        unsafe {
            daily_core_call_client_update_subscriptions(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                participant_settings_ptr,
                profile_settings_ptr,
            );

            if !participant_settings_ptr.is_null() {
                let _ = CString::from_raw(participant_settings_ptr);
            }
            if !profile_settings_ptr.is_null() {
                let _ = CString::from_raw(profile_settings_ptr);
            }
        }
    }

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

    pub fn update_subscription_profiles(&mut self, py_profile_settings: PyObject) {
        let profile_settings: HashMap<String, DictValue> =
            Python::with_gil(|py| py_profile_settings.extract(py).unwrap());

        let profile_settings_string = serde_json::to_string(&profile_settings).unwrap();
        let profile_settings_ptr = CString::new(profile_settings_string)
            .expect("invalid profile settings string")
            .into_raw();

        unsafe {
            daily_core_call_client_update_subscription_profiles(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                profile_settings_ptr,
            );

            let _ = CString::from_raw(profile_settings_ptr);
        }
    }

    pub fn update_permissions(&mut self, py_permissions: PyObject) {
        let permissions: HashMap<String, DictValue> =
            Python::with_gil(|py| py_permissions.extract(py).unwrap());

        let permissions_string = serde_json::to_string(&permissions).unwrap();
        let permissions_ptr = CString::new(permissions_string)
            .expect("invalid permissions string")
            .into_raw();

        unsafe {
            daily_core_call_client_update_permissions(
                self.call_client.as_mut(),
                GLOBAL_CONTEXT.as_ref().unwrap().next_request_id(),
                permissions_ptr,
            );

            let _ = CString::from_raw(permissions_ptr);
        }
    }

    #[pyo3(signature = (participant_id, callback, video_source = "camera", color_format = "RGBA32"))]
    pub fn set_video_renderer(
        &mut self,
        participant_id: &str,
        callback: PyObject,
        video_source: &str,
        color_format: &str,
    ) -> PyResult<()> {
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

        unsafe {
            let result = daily_core_call_client_set_participant_video_renderer(
                self.call_client.as_mut(),
                participant_ptr,
                video_source_ptr,
                color_format_ptr,
                video_renderer,
            );

            let _ = CString::from_raw(participant_ptr);
            let _ = CString::from_raw(video_source_ptr);
            let _ = CString::from_raw(color_format_ptr);

            if result {
                Ok(())
            } else {
                Err(exceptions::PyRuntimeError::new_err(
                    "unable to set video renderer",
                ))
            }
        }
    }
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

        if let Err(error) = callback_ctx.callback.call1(py, args) {
            error.write_unraisable(py, None);
        }
    });
}

impl Drop for PyCallClient {
    fn drop(&mut self) {
        self.leave();
    }
}
