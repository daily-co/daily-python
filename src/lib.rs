#[macro_use]
extern crate lazy_static;

pub(crate) mod call_client;
pub(crate) mod context;
pub(crate) mod media;
pub(crate) mod util;

use call_client::{PyCallClient, PyEventHandler};
use context::GLOBAL_CONTEXT;
use media::{
    PyAudioData, PyCustomAudioSource, PyNativeVad, PyVideoFrame, PyVirtualCameraDevice,
    PyVirtualMicrophoneDevice, PyVirtualSpeakerDevice,
};

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
    // Probably nothing to do here since our microphone device is already
    // properly selected during getUserMedia.
}

unsafe extern "C" fn get_audio_device(_delegate: *mut libc::c_void) -> *const libc::c_char {
    GLOBAL_CONTEXT.get_selected_microphone_device()
}

unsafe extern "C" fn get_enumerated_devices(_delegate: *mut libc::c_void) -> *mut libc::c_char {
    GLOBAL_CONTEXT.get_enumerated_devices()
}

unsafe extern "C" fn get_user_media(
    _delegate: *mut libc::c_void,
    peer_connection_factory: *mut WebrtcPeerConnectionFactory,
    signaling_thread: *mut WebrtcThread,
    worker_thread: *mut WebrtcThread,
    network_thread: *mut WebrtcThread,
    constraints: *const libc::c_char,
) -> *mut libc::c_void {
    GLOBAL_CONTEXT.get_user_media(
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
    GLOBAL_CONTEXT.create_audio_device_module(task_queue_factory)
}

/// This class is used to initialize the SDK and create virtual devices.
#[pyclass(name = "Daily", module = "daily")]
struct PyDaily;

#[pymethods]
impl PyDaily {
    /// Initializes the SDK. This function needs to be called before anything
    /// else, usually done at the application startup.
    ///
    /// :param int worker_threads: Number of internal worker threads. Increasing this number might be necessary if the application needs to create a large number of concurrent call clients
    #[staticmethod]
    #[pyo3(signature = (worker_threads = 2))]
    pub fn init(worker_threads: usize) {
        unsafe {
            daily_core_set_log_level(LogLevel::Off);
        }

        let library_cstr = CString::new(DAILY_PYTHON_NAME).expect("invalid library string");
        let version_cstr = CString::new(DAILY_PYTHON_VERSION).expect("invalid version string");
        let os_cstr = CString::new(env::consts::OS).expect("invalid OS string");

        let about_client = NativeAboutClient::new(
            library_cstr.as_ptr(),
            version_cstr.as_ptr(),
            os_cstr.as_ptr(),
            ptr::null(),
            library_cstr.as_ptr(), // TODO replace with app name when implementing Banuba
        );

        let context_delegate =
            NativeContextDelegate::new(NativeContextDelegatePtr::new(ptr::null_mut()));

        let webrtc_delegate = NativeWebRtcContextDelegate::new(
            NativeWebRtcContextDelegatePtr::new(ptr::null_mut()),
            NativeWebRtcContextDelegateFns::new(
                get_user_media,
                None,
                get_enumerated_devices,
                Some(create_audio_device_module),
                None,
                None,
                None,
                None,
                get_audio_device,
                set_audio_device,
                None,
            ),
        );

        daily_core_context_create_with_threads(
            context_delegate,
            webrtc_delegate,
            about_client,
            worker_threads,
        );
    }

    /// Deallocates SDK resources. This is usually called when shutting down the
    /// application.
    #[staticmethod]
    pub fn deinit() {
        // TODO(aleix): We need to make sure all clients leave before doing this
        // otherwise we might crash.
        unsafe { daily_core_context_destroy() };
    }

    /// Creates a new virtual camera device. Camera devices are used to
    /// send video (i.e. video frames) into the meeting.
    ///
    /// :param str device_name: The virtual camera device name
    /// :param int width: Resolution width
    /// :param int height: Resolution height
    /// :param str color_format: The color format of the frames that will be written to the camera device. See :ref:`ColorFormat`
    ///
    /// :return: A new virtual camera device
    /// :rtype: :class:`daily.VirtualCameraDevice`
    #[staticmethod]
    #[pyo3(signature = (device_name, width, height, color_format = "RGBA"))]
    pub fn create_camera_device(
        device_name: &str,
        width: u32,
        height: u32,
        color_format: &str,
    ) -> PyResult<PyVirtualCameraDevice> {
        GLOBAL_CONTEXT.create_camera_device(device_name, width, height, color_format)
    }

    /// Creates a new virtual speaker device. Speaker devices are used to
    /// receive audio (i.e. read audio frames) from the meeting.
    ///
    /// There can only be one speaker device per application and it needs to be
    /// set with :func:`select_speaker_device`.
    ///
    /// :param str device_name: The virtual speaker device name
    /// :param int sample_rate: Sample rate
    /// :param int channels: Number of channels (2 for stereo, 1 for mono)
    /// :param bool non_blocking: Whether the speaker will be blocking or non-blocking
    ///
    /// :return: A new virtual speaker device
    /// :rtype: :class:`daily.VirtualSpeakerDevice`
    #[staticmethod]
    #[pyo3(signature = (device_name, sample_rate = 16000, channels = 1, non_blocking = false))]
    pub fn create_speaker_device(
        device_name: &str,
        sample_rate: u32,
        channels: u8,
        non_blocking: bool,
    ) -> PyResult<PyVirtualSpeakerDevice> {
        GLOBAL_CONTEXT.create_speaker_device(device_name, sample_rate, channels, non_blocking)
    }

    /// Creates a new virtual microphone device. Microphone devices are used to
    /// send audio (i.e. write audio frames) to the meeting.
    ///
    /// Microphone devices are selected with :func:`CallClient.update_inputs`.
    ///
    /// :param str device_name: The virtual microphone device name. This can be used as a `deviceId` when configuring the call client inputs
    /// :param int sample_rate: Sample rate
    /// :param int channels: Number of channels (2 for stereo, 1 for mono)
    /// :param bool non_blocking: Whether the microphone will be blocking or non-blocking
    ///
    /// :return: A new virtual microphone device
    /// :rtype: :class:`daily.VirtualMicrophoneDevice`
    #[staticmethod]
    #[pyo3(signature = (device_name, sample_rate = 16000, channels = 1, non_blocking = false))]
    pub fn create_microphone_device(
        device_name: &str,
        sample_rate: u32,
        channels: u8,
        non_blocking: bool,
    ) -> PyResult<PyVirtualMicrophoneDevice> {
        GLOBAL_CONTEXT.create_microphone_device(device_name, sample_rate, channels, non_blocking)
    }

    /// Selects one of the previously created virtual speaker devices to be the
    /// main system speaker. Note that there can only be one speaker selected at
    /// a time. Also, if there are multiple participants in the meeting, the
    /// audio from all the participants will be mixed and that's the audio that
    /// is received in the speaker.
    ///
    /// :param str device_name: The name of the virtual speaker device to select
    #[staticmethod]
    pub fn select_speaker_device(device_name: &str) -> PyResult<()> {
        GLOBAL_CONTEXT.select_speaker_device(device_name)
    }

    /// Creates a new VAD analyzer. VADs are used to detect speech from an audio
    /// stream.
    ///
    /// :param int reset_period_ms: The period in milliseconds after the VAD is internally reset
    /// :param int sample_rate: Sample rate of the incoming audio frames
    /// :param int channels: Number of channels (2 for stereo, 1 for mono) of the incoming audio frames
    ///
    /// :return: A new VAD
    /// :rtype: :class:`daily.NativeVad`
    #[staticmethod]
    #[pyo3(signature = (reset_period_ms = 1000, sample_rate = 16000, channels = 1))]
    pub fn create_native_vad(
        reset_period_ms: u32,
        sample_rate: u32,
        channels: u8,
    ) -> PyResult<PyNativeVad> {
        GLOBAL_CONTEXT.create_native_vad(reset_period_ms, sample_rate, channels)
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn daily(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAudioData>()?;
    m.add_class::<PyCallClient>()?;
    m.add_class::<PyCustomAudioSource>()?;
    m.add_class::<PyDaily>()?;
    m.add_class::<PyEventHandler>()?;
    m.add_class::<PyNativeVad>()?;
    m.add_class::<PyVideoFrame>()?;
    m.add_class::<PyVirtualCameraDevice>()?;
    m.add_class::<PyVirtualMicrophoneDevice>()?;
    m.add_class::<PyVirtualSpeakerDevice>()?;
    Ok(())
}
