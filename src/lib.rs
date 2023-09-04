pub mod call_client;
pub mod context;
pub mod custom_microphone_device;
pub mod custom_speaker_device;
pub mod dict;
pub mod event;
pub mod event_handler;
pub mod video_frame;

use call_client::PyCallClient;
use context::{DailyContext, GLOBAL_CONTEXT};
use custom_microphone_device::PyCustomMicrophoneDevice;
use custom_speaker_device::PyCustomSpeakerDevice;
use event_handler::PyEventHandler;
use video_frame::PyVideoFrame;

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
    device_id: *const libc::c_char,
) {
    let device_cstr = CString::from_raw(device_id as *mut _);

    let device = device_cstr.clone().into_string().unwrap();

    let result = GLOBAL_CONTEXT
        .as_mut()
        .unwrap()
        .select_microphone_device(device.as_str());

    Python::with_gil(|py| {
        if let Err(error) = result {
            error.write_unraisable(py, None);
        }
    });

    // Release pointer and avoid double-free.
    let _ = device_cstr.into_raw();
}

unsafe extern "C" fn get_audio_device(_delegate: *mut libc::c_void) -> *const libc::c_char {
    GLOBAL_CONTEXT
        .as_ref()
        .unwrap()
        .get_selected_microphone_device()
}

unsafe extern "C" fn get_enumerated_devices(_delegate: *mut libc::c_void) -> *mut libc::c_char {
    GLOBAL_CONTEXT.as_ref().unwrap().get_enumerated_devices()
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

/// This class is used to initialize the SDK and create custom devices.
#[pyclass(name = "Daily", module = "daily")]
struct PyDaily;

#[pymethods]
impl PyDaily {
    /// Initializes the SDK. This function needs to be called before anything
    /// else, usually done at the application startup.
    ///
    /// :param bool custom_devices: If True the default system devices (camera, speaker and microphone) will be used. Otherwise, custom  devices can be registered
    /// :param int worker_threads: Number of internal worker threads. Increasing this number might be necessary if the application needs to create a large number of concurrent call clients
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

    /// Deallocates SDK resources. This is usually called when shutting down the
    /// application.
    #[staticmethod]
    pub fn deinit() {
        // TODO(aleix): We need to make sure all clients leave before doing this
        // otherwise we might crash.
        unsafe { daily_core_context_destroy() };
    }

    /// Creates a new custom speaker device. New custom speaker devices can only
    /// be created if `custom_devices` was set to True when calling
    /// :func:`init`, otherwise the system audio devices will be used. Speaker
    /// devices are used to receive audio (i.e. read audio samples) from the
    /// meeting.
    ///
    /// There can only be one speaker device per application and it needs to be
    /// set with :func:`select_custom_speaker_device`.
    ///
    /// :param str device_name: The custom speaker device name
    /// :param int sample_rate: Sample rate
    /// :param int channels: Number of channels (2 for stereo, 1 for mono)
    ///
    /// :return: A new custom speaker device
    /// :rtype: :class:`daily.CustomSpeakerDevice`
    #[staticmethod]
    #[pyo3(signature = (device_name, sample_rate = 16000, channels = 2))]
    pub fn create_speaker_device(
        device_name: &str,
        sample_rate: u32,
        channels: u8,
    ) -> PyResult<PyCustomSpeakerDevice> {
        unsafe {
            GLOBAL_CONTEXT.as_mut().unwrap().create_speaker_device(
                device_name,
                sample_rate,
                channels,
            )
        }
    }

    /// Creates a new custom microphone device. New custom microphone devices
    /// can only be created if `custom_devices` was set to True when calling
    /// :func:`init`, otherwise the system audio devices will be
    /// used. Microphone devices are used to send audio (i.e. write audio
    /// samples) to the meeting.
    ///
    /// Microphone devices are selected with :func:`CallClient.update_inputs`.
    ///
    /// :param str device_name: The custom microphone device name. This can be used as a `deviceId` when configuring the call client inputs
    /// :param int sample_rate: Sample rate
    /// :param int channels: Number of channels (2 for stereo, 1 for mono)
    ///
    /// :return: A new custom microphone device
    /// :rtype: :class:`daily.CustomMicrophoneDevice`
    #[staticmethod]
    #[pyo3(signature = (device_name, sample_rate = 16000, channels = 2))]
    pub fn create_microphone_device(
        device_name: &str,
        sample_rate: u32,
        channels: u8,
    ) -> PyResult<PyCustomMicrophoneDevice> {
        unsafe {
            GLOBAL_CONTEXT.as_mut().unwrap().create_microphone_device(
                device_name,
                sample_rate,
                channels,
            )
        }
    }

    /// Selects one of the previously created custom speaker devices to be the
    /// main system speaker. Note that there can only be one speaker selected at
    /// a time. Also, if there are multiple participants in the meeting, the
    /// audio from all the participants will be mixed and that's the audio that
    /// is received in the speaker.
    ///
    /// :param str device_name: The name of the custom speaker device to select
    #[staticmethod]
    pub fn select_speaker_device(device_name: &str) -> PyResult<()> {
        unsafe {
            GLOBAL_CONTEXT
                .as_mut()
                .unwrap()
                .select_speaker_device(device_name)
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn daily(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyDaily>()?;
    m.add_class::<PyCallClient>()?;
    m.add_class::<PyCustomSpeakerDevice>()?;
    m.add_class::<PyCustomMicrophoneDevice>()?;
    m.add_class::<PyEventHandler>()?;
    m.add_class::<PyVideoFrame>()?;
    Ok(())
}
