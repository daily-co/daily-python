pub mod call_client;
pub mod context;
pub mod custom_audio_device;
pub mod dict;

use call_client::PyCallClient;
use context::{DailyContext, GLOBAL_CONTEXT};
use custom_audio_device::PyCustomAudioDevice;
use dict::DictValue;

use std::env;
use std::ffi::CString;
use std::ptr;

use daily_core::prelude::{
    daily_core_context_create_with_threads, daily_core_context_destroy, daily_core_set_log_level,
    LogLevel, NativeAboutClient, NativeContextDelegate, NativeContextDelegatePtr,
    NativeRawWebRtcContextDelegate, NativeWebRtcContextDelegate, NativeWebRtcContextDelegateFns,
    NativeWebRtcContextDelegatePtr, WebrtcAudioDeviceModule, WebrtcPeerConnectionFactory,
    WebrtcTaskQueueFactory, WebrtcThread,
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
    peer_connection_factory: *mut WebrtcPeerConnectionFactory,
    signaling_thread: *mut WebrtcThread,
    worker_thread: *mut WebrtcThread,
    network_thread: *mut WebrtcThread,
    constraints: *const libc::c_char,
) -> *mut libc::c_void {
    GLOBAL_CONTEXT.as_mut().unwrap().get_user_media(
        peer_connection_factory,
        signaling_thread,
        worker_thread,
        network_thread,
        constraints,
    )
}

unsafe extern "C" fn create_audio_device_module(
    _delegate: *mut NativeRawWebRtcContextDelegate,
    task_queue_factory: *mut WebrtcTaskQueueFactory,
) -> *mut WebrtcAudioDeviceModule {
    GLOBAL_CONTEXT
        .as_mut()
        .unwrap()
        .create_audio_device_module(task_queue_factory)
}

#[pyclass(name = "Daily", module = "daily")]
struct PyDaily;

#[pymethods]
impl PyDaily {
    #[staticmethod]
    #[pyo3(signature = (custom_devices = false, worker_threads = 2))]
    pub fn init(custom_devices: bool, worker_threads: usize) {
        unsafe {
            GLOBAL_CONTEXT = Some(DailyContext::new());
            daily_core_set_log_level(LogLevel::Off);
        }

        let library_ptr = CString::new(DAILY_PYTHON_NAME)
            .expect("invalid library string")
            .into_raw();
        let version_ptr = CString::new(DAILY_PYTHON_VERSION)
            .expect("invalid version string")
            .into_raw();
        let os_ptr = CString::new(env::consts::OS)
            .expect("invalid OS string")
            .into_raw();

        let about_client = NativeAboutClient {
            library: library_ptr,
            version: version_ptr,
            operating_system: os_ptr,
            operating_system_version: ptr::null(),
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
                create_audio_device_module: if custom_devices {
                    Some(create_audio_device_module)
                } else {
                    None
                },
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

        unsafe {
            let _ = CString::from_raw(library_ptr);
            let _ = CString::from_raw(version_ptr);
            let _ = CString::from_raw(os_ptr);
        }
    }

    #[staticmethod]
    #[pyo3(signature = (device_name, play_sample_rate = 16000, play_channels = 2, recording_sample_rate = 16000, recording_channels = 2))]
    pub fn create_custom_audio_device(
        device_name: &str,
        play_sample_rate: u32,
        play_channels: u8,
        recording_sample_rate: u32,
        recording_channels: u8,
    ) -> PyResult<PyCustomAudioDevice> {
        unsafe {
            GLOBAL_CONTEXT.as_mut().unwrap().create_custom_audio_device(
                device_name,
                play_sample_rate,
                play_channels,
                recording_sample_rate,
                recording_channels,
            )
        }
    }

    #[staticmethod]
    pub fn select_custom_audio_device(device_name: &str) -> PyResult<()> {
        unsafe {
            GLOBAL_CONTEXT
                .as_mut()
                .unwrap()
                .select_custom_audio_device(device_name)
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
    m.add_class::<PyCustomAudioDevice>()?;
    Ok(())
}
