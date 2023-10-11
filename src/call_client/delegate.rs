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

use super::event::{
    args_from_event, method_name_from_event_action, request_id_from_event, update_inner_values,
    Event,
};

use crate::PyVideoFrame;

type PyCallClientDelegateOnEventFn =
    unsafe fn(py: Python<'_>, delegate_ctx: &DelegateContext, event: &Event);

type PyCallClientDelegateOnVideoFrameFn = unsafe fn(
    py: Python<'_>,
    delegate_ctx: &DelegateContext,
    renderer_id: u64,
    peer_id: *const libc::c_char,
    frame: *const NativeVideoFrame,
);

#[derive(Clone)]
pub(crate) struct PyCallClientDelegateFns {
    pub(crate) on_event: Option<PyCallClientDelegateOnEventFn>,
    pub(crate) on_video_frame: Option<PyCallClientDelegateOnVideoFrameFn>,
}

#[derive(Clone)]
pub(crate) struct PyCallClientInner {
    pub(crate) delegates: Arc<Mutex<PyCallClientDelegateFns>>,
    pub(crate) completions: Arc<Mutex<HashMap<u64, PyObject>>>,
    pub(crate) video_renderers: Arc<Mutex<HashMap<u64, PyObject>>>,
    // Non-blocking updates
    pub(crate) active_speaker: Arc<Mutex<PyObject>>,
    pub(crate) inputs: Arc<Mutex<PyObject>>,
    pub(crate) participant_counts: Arc<Mutex<PyObject>>,
    pub(crate) publishing: Arc<Mutex<PyObject>>,
    pub(crate) subscriptions: Arc<Mutex<PyObject>>,
    pub(crate) subscription_profiles: Arc<Mutex<PyObject>>,
    pub(crate) network_stats: Arc<Mutex<PyObject>>,
}

#[derive(Clone)]
pub(crate) struct DelegateContext {
    pub(crate) inner: Arc<PyCallClientInner>,
    pub(crate) event_handler_callback: Option<PyObject>,
}

#[derive(Clone)]
pub(crate) struct DelegateContextPtr {
    pub(crate) ptr: *const DelegateContext,
}

unsafe impl Send for DelegateContextPtr {}

pub(crate) unsafe extern "C" fn on_event_native(
    delegate: *mut libc::c_void,
    event_json: *const libc::c_char,
    _json_len: isize,
) {
    // Acquire the GIL before checking if there's a delegate available. If
    // PyCallClient is dropping it will cleanup the delegates and will
    // temporarily release the GIL so we can proceed.
    Python::with_gil(|py| {
        let delegate_ctx_ptr = delegate as *const DelegateContext;

        // We increment the reference count because otherwise it will get dropped
        // when Arc::from_raw() takes ownership, and we still want to keep the
        // delegate pointer around.
        Arc::increment_strong_count(delegate_ctx_ptr);

        let delegate_ctx = Arc::from_raw(delegate_ctx_ptr);

        let delegate = delegate_ctx
            .inner
            .delegates
            .clone()
            .lock()
            .unwrap()
            .on_event;

        if let Some(delegate) = delegate {
            let event_string = CStr::from_ptr(event_json).to_string_lossy().into_owned();
            let event = serde_json::from_str::<Event>(event_string.as_str()).unwrap();

            delegate(py, &delegate_ctx, &event);
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
        let delegate_ctx_ptr = delegate as *const DelegateContext;

        // We increment the reference count because otherwise it will get dropped
        // when Arc::from_raw() takes ownership, and we still want to keep the
        // delegate pointer around.
        Arc::increment_strong_count(delegate_ctx_ptr);

        let delegate_ctx = Arc::from_raw(delegate_ctx_ptr);

        let delegate = delegate_ctx
            .inner
            .delegates
            .clone()
            .lock()
            .unwrap()
            .on_video_frame;

        if let Some(delegate) = delegate {
            delegate(py, &delegate_ctx, renderer_id, peer_id, frame);
        }
    });
}

pub(crate) unsafe fn on_event(py: Python<'_>, delegate_ctx: &DelegateContext, event: &Event) {
    match event.action.as_str() {
        "request-completed" => {
            if let Some(request_id) = request_id_from_event(event) {
                if let Some(delegate) = delegate_ctx
                    .inner
                    .completions
                    .lock()
                    .unwrap()
                    .remove(&request_id)
                {
                    if let Some(args) = args_from_event(event) {
                        let py_args = PyTuple::new(py, args);

                        if let Err(error) = delegate.call1(py, py_args) {
                            error.write_unraisable(py, None);
                        }
                    }
                }
            }
        }
        action => {
            if let Some(method_name) = method_name_from_event_action(action) {
                if let Some(args) = args_from_event(event) {
                    if let Some(callback) = &delegate_ctx.event_handler_callback {
                        let py_args = PyTuple::new(py, args.clone());

                        if let Err(error) = callback.call_method1(py, method_name, py_args) {
                            error.write_unraisable(py, None);
                        }
                    }

                    // Update inner values asynchronously.
                    update_inner_values(py, delegate_ctx, action, args);
                }
            }
        }
    }
}

pub(crate) unsafe fn on_video_frame(
    py: Python<'_>,
    delegate_ctx: &DelegateContext,
    renderer_id: u64,
    peer_id: *const libc::c_char,
    frame: *const NativeVideoFrame,
) {
    if let Some(callback) = delegate_ctx
        .inner
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
