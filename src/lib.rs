pub mod call_client;
pub mod context;
pub mod dict;
pub mod event;
pub mod event_handler;
pub mod video_frame;
pub mod virtual_camera_device;
pub mod virtual_microphone_device;
pub mod virtual_speaker_device;

use call_client::PyCallClient;
use context::{DailyContext, GLOBAL_CONTEXT};
use event_handler::PyEventHandler;
use video_frame::PyVideoFrame;
use virtual_camera_device::PyVirtualCameraDevice;
use virtual_microphone_device::PyVirtualMicrophoneDevice;
use virtual_speaker_device::PyVirtualSpeakerDevice;

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
            GLOBAL_CONTEXT = Some(DailyContext::new());
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
        );

        let context_delegate =
            NativeContextDelegate::new(NativeContextDelegatePtr::new(ptr::null_mut()));

        let webrtc_delegate = NativeWebRtcContextDelegate::new(
            NativeWebRtcContextDelegatePtr::new(ptr::null_mut()),
            NativeWebRtcContextDelegateFns::new(
                get_user_media,
                get_enumerated_devices,
                Some(create_audio_device_module),
                None,
                None,
                None,
                None,
                get_audio_device,
                set_audio_device,
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
    #[pyo3(signature = (device_name, width, height, color_format = "RGBA32"))]
    pub fn create_camera_device(
        device_name: &str,
        width: u32,
        height: u32,
        color_format: &str,
    ) -> PyResult<PyVirtualCameraDevice> {
        unsafe {
            GLOBAL_CONTEXT.as_mut().unwrap().create_camera_device(
                device_name,
                width,
                height,
                color_format,
            )
        }
    }

    /// Creates a new virtual speaker device. Speaker devices are used to
    /// receive audio (i.e. read audio samples) from the meeting.
    ///
    /// There can only be one speaker device per application and it needs to be
    /// set with :func:`select_speaker_device`.
    ///
    /// :param str device_name: The virtual speaker device name
    /// :param int sample_rate: Sample rate
    /// :param int channels: Number of channels (2 for stereo, 1 for mono)
    ///
    /// :return: A new virtual speaker device
    /// :rtype: :class:`daily.VirtualSpeakerDevice`
    #[staticmethod]
    #[pyo3(signature = (device_name, sample_rate = 16000, channels = 1))]
    pub fn create_speaker_device(
        device_name: &str,
        sample_rate: u32,
        channels: u8,
    ) -> PyResult<PyVirtualSpeakerDevice> {
        unsafe {
            GLOBAL_CONTEXT.as_mut().unwrap().create_speaker_device(
                device_name,
                sample_rate,
                channels,
            )
        }
    }

    /// Creates a new virtual microphone device. Microphone devices are used to
    /// send audio (i.e. write audio samples) to the meeting.
    ///
    /// Microphone devices are selected with :func:`CallClient.update_inputs`.
    ///
    /// :param str device_name: The virtual microphone device name. This can be used as a `deviceId` when configuring the call client inputs
    /// :param int sample_rate: Sample rate
    /// :param int channels: Number of channels (2 for stereo, 1 for mono)
    ///
    /// :return: A new virtual microphone device
    /// :rtype: :class:`daily.VirtualMicrophoneDevice`
    #[staticmethod]
    #[pyo3(signature = (device_name, sample_rate = 16000, channels = 1))]
    pub fn create_microphone_device(
        device_name: &str,
        sample_rate: u32,
        channels: u8,
    ) -> PyResult<PyVirtualMicrophoneDevice> {
        unsafe {
            GLOBAL_CONTEXT.as_mut().unwrap().create_microphone_device(
                device_name,
                sample_rate,
                channels,
            )
        }
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
    m.add_class::<PyEventHandler>()?;
    m.add_class::<PyVideoFrame>()?;
    m.add_class::<PyVirtualCameraDevice>()?;
    m.add_class::<PyVirtualMicrophoneDevice>()?;
    m.add_class::<PyVirtualSpeakerDevice>()?;
    Ok(())
}
