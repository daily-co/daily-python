use std::{
    collections::HashMap,
    ffi::CStr,
    sync::{Arc, Mutex},
};

use pyo3::{
    prelude::*,
    types::{PyBytes, PyTuple},
};

use daily_core::prelude::*;

use crate::{
    event::{args_from_event, method_name_from_event, request_id_from_event, Event},
    PyVideoFrame,
};

type PyCallClientDelegateOnEventFn =
    unsafe fn(py: Python<'_>, callback_ctx: &CallbackContext, event: &Event);

type PyCallClientDelegateOnVideoFrameFn = unsafe fn(
    py: Python<'_>,
    callback_ctx: &CallbackContext,
    renderer_id: u64,
    peer_id: *const libc::c_char,
    frame: *const NativeVideoFrame,
);

#[derive(Clone, Debug)]
pub(crate) struct PyCallClientDelegateFns {
    pub(crate) on_event: Option<PyCallClientDelegateOnEventFn>,
    pub(crate) on_video_frame: Option<PyCallClientDelegateOnVideoFrameFn>,
}

pub(crate) struct CallbackContext {
    pub(crate) delegates: Arc<Mutex<PyCallClientDelegateFns>>,
    pub(crate) event_handler_callback: Option<PyObject>,
    pub(crate) completions: Arc<Mutex<HashMap<u64, PyObject>>>,
    pub(crate) video_renderers: Arc<Mutex<HashMap<u64, PyObject>>>,
}

#[derive(Clone)]
pub(crate) struct CallbackContextPtr {
    pub(crate) ptr: *const CallbackContext,
}

unsafe impl Send for CallbackContextPtr {}
pub(crate) unsafe extern "C" fn on_event_native(
    delegate: *mut libc::c_void,
    event_json: *const libc::c_char,
    _json_len: isize,
) {
    // Acquire the GIL before checking if there's a delegate available. If
    // PyCallClient is dropping it will cleanup the delegates and will
    // temporarily release the GIL so we can proceed.
    Python::with_gil(|py| {
        let callback_ctx_ptr = delegate as *const CallbackContext;

        // We increment the reference count because otherwise it will get dropped
        // when Arc::from_raw() takes ownership, and we still want to keep the
        // delegate pointer around.
        Arc::increment_strong_count(callback_ctx_ptr);

        let callback_ctx = Arc::from_raw(callback_ctx_ptr);

        let delegate = callback_ctx.delegates.clone().lock().unwrap().on_event;

        if let Some(delegate) = delegate {
            let event_string = CStr::from_ptr(event_json).to_string_lossy().into_owned();
            let event = serde_json::from_str::<Event>(event_string.as_str()).unwrap();

            delegate(py, &callback_ctx, &event);
        }
    });
}

pub(crate) unsafe extern "C" fn on_video_frame_native(
    delegate: *mut libc::c_void,
    renderer_id: u64,
    peer_id: *const libc::c_char,
    frame: *const NativeVideoFrame,
) {
    // Acquire the GIL before checking if there's a delegate available. If
    // PyCallClient is dropping it will cleanup the delegates and will
    // temporarily release the GIL so we can proceed.
    Python::with_gil(|py| {
        let callback_ctx_ptr = delegate as *const CallbackContext;

        // We increment the reference count because otherwise it will get dropped
        // when Arc::from_raw() takes ownership, and we still want to keep the
        // delegate pointer around.
        Arc::increment_strong_count(callback_ctx_ptr);

        let callback_ctx = Arc::from_raw(callback_ctx_ptr);

        let delegate = callback_ctx
            .delegates
            .clone()
            .lock()
            .unwrap()
            .on_video_frame;

        if let Some(delegate) = delegate {
            delegate(py, &callback_ctx, renderer_id, peer_id, frame);
        }
    });
}

pub(crate) unsafe fn on_event(py: Python<'_>, callback_ctx: &CallbackContext, event: &Event) {
    match event.action.as_str() {
        "request-completed" => {
            if let Some(request_id) = request_id_from_event(event) {
                if let Some(callback) = callback_ctx.completions.lock().unwrap().remove(&request_id)
                {
                    if let Some(args) = args_from_event(event) {
                        let py_args = PyTuple::new(py, args);

                        if let Err(error) = callback.call1(py, py_args) {
                            error.write_unraisable(py, None);
                        }
                    }
                }
            }
        }
        _ => {
            if let Some(callback) = &callback_ctx.event_handler_callback {
                if let Some(method_name) = method_name_from_event(event) {
                    if let Some(args) = args_from_event(event) {
                        let py_args = PyTuple::new(py, args);

                        if let Err(error) = callback.call_method1(py, method_name, py_args) {
                            error.write_unraisable(py, None);
                        }
                    }
                }
            }
        }
    }
}

pub(crate) unsafe fn on_video_frame(
    py: Python<'_>,
    callback_ctx: &CallbackContext,
    renderer_id: u64,
    peer_id: *const libc::c_char,
    frame: *const NativeVideoFrame,
) {
    if let Some(callback) = callback_ctx
        .video_renderers
        .clone()
        .lock()
        .unwrap()
        .get(&renderer_id)
    {
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
}
