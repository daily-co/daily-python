pub mod call_client;
pub mod context;
pub mod dict;

use call_client::PyCallClient;
use context::{DailyContext, GLOBAL_CONTEXT};
use dict::DictValue;

use std::env;
use std::ptr;

use daily_core::prelude::{
    daily_core_context_create_with_threads, daily_core_context_destroy, daily_core_set_log_level,
    LogLevel, NativeAboutClient, NativeContextDelegate, NativeContextDelegatePtr,
    NativeWebRtcContextDelegate, NativeWebRtcContextDelegateFns, NativeWebRtcContextDelegatePtr,
    WebrtcPeerConnectionFactory, WebrtcThread,
};

use pyo3::prelude::*;

const DAILY_PYTHON_NAME: &str = "daily-python";
const DAILY_PYTHON_VERSION: &str = env!("CARGO_PKG_VERSION");

unsafe extern "C" fn set_audio_device(
    _delegate: *mut libc::c_void,
    _device_id: *const libc::c_char,
) {
}

unsafe extern "C" fn get_audio_device(_delegate: *mut libc::c_void) -> *const libc::c_char {
    concat!("", "\0").as_ptr() as *const libc::c_char
}

unsafe extern "C" fn get_enumerated_devices(_delegate: *mut libc::c_void) -> *mut libc::c_char {
    concat!("[]", "\0").as_ptr() as *mut libc::c_char
}

unsafe extern "C" fn get_user_media(
    _delegate: *mut libc::c_void,
    _peer_connection_factory: *mut WebrtcPeerConnectionFactory,
    _signaling_thread: *mut WebrtcThread,
    _worker_thread: *mut WebrtcThread,
    _network_thread: *mut WebrtcThread,
    _constraints: *const libc::c_char,
) -> *mut libc::c_void {
    GLOBAL_CONTEXT.as_ref().unwrap().media_stream().as_ptr() as *mut libc::c_void
}

#[pyclass(name = "Daily", module = "daily")]
struct PyDaily;

#[pymethods]
impl PyDaily {
    #[staticmethod]
    #[pyo3(signature = (worker_threads = 2))]
    pub fn init(worker_threads: usize) {
        unsafe { daily_core_set_log_level(LogLevel::Off) };

        let about_client = NativeAboutClient {
            library: DAILY_PYTHON_NAME.as_ptr() as *const libc::c_char,
            version: DAILY_PYTHON_VERSION.as_ptr() as *const libc::c_char,
            operating_system: env::consts::OS.as_ptr() as *const libc::c_char,
            operating_system_version: "".as_ptr() as *const libc::c_char,
        };

        let context_delegate = NativeContextDelegate {
            ptr: NativeContextDelegatePtr(ptr::null_mut()),
        };

        let webrtc_delegate = NativeWebRtcContextDelegate {
            ptr: NativeWebRtcContextDelegatePtr(ptr::null_mut()),
            fns: NativeWebRtcContextDelegateFns {
                get_audio_device,
                set_audio_device,
                get_enumerated_devices,
                get_user_media,
                create_audio_device_module: None,
                create_video_decoder_factory: None,
                create_video_encoder_factory: None,
                create_audio_decoder_factory: None,
                create_audio_encoder_factory: None,
            },
        };

        daily_core_context_create_with_threads(
            context_delegate,
            webrtc_delegate,
            about_client,
            worker_threads,
        );

        if let Ok(context) = DailyContext::new() {
            unsafe {
                GLOBAL_CONTEXT = Some(context);
            }
        }
    }

    #[staticmethod]
    pub fn deinit() {
        // TODO(aleix): We need to make sure all clients leave before doing this
        // otherwise we might crash.
        unsafe { daily_core_context_destroy() };
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn daily(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyDaily>()?;
    m.add_class::<PyCallClient>()?;
    Ok(())
}
