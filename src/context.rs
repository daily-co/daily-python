use std::ffi::{CStr, CString};
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::PyNativeVad;
use crate::PyVirtualCameraDevice;
use crate::PyVirtualMicrophoneDevice;
use crate::PyVirtualSpeakerDevice;

use webrtc_daily::sys::{
    color_format::ColorFormat, device_manager::NativeDeviceManager, vad::NativeWebrtcVad,
    virtual_camera_device::NativeVirtualCameraDevice,
    virtual_microphone_device::NativeVirtualMicrophoneDevice,
    virtual_speaker_device::NativeVirtualSpeakerDevice,
};

use daily_core::prelude::{
    daily_core_context_create_audio_device_module, daily_core_context_create_device_manager,
    daily_core_context_create_vad, daily_core_context_create_virtual_camera_device,
    daily_core_context_create_virtual_microphone_device,
    daily_core_context_create_virtual_speaker_device,
    daily_core_context_device_manager_enumerated_devices,
    daily_core_context_device_manager_get_user_media,
    daily_core_context_get_selected_microphone_device, daily_core_context_select_speaker_device,
    WebrtcAudioDeviceModule, WebrtcPeerConnectionFactory, WebrtcTaskQueueFactory, WebrtcThread,
};

use pyo3::exceptions;
use pyo3::prelude::*;

lazy_static! {
    pub(crate) static ref GLOBAL_CONTEXT: DailyContext = DailyContext::new();
}

pub(crate) struct DailyContext {
    request_id: AtomicU64,
    device_manager: NativeDeviceManager,
}

impl DailyContext {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let device_manager_ptr = unsafe { daily_core_context_create_device_manager() };

        let device_manager = NativeDeviceManager::from(device_manager_ptr as *mut _);

        Self {
            device_manager,
            request_id: AtomicU64::new(0),
        }
    }

    pub fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn get_enumerated_devices(&self) -> *mut libc::c_char {
        const EMPTY: &[u8] = b"[]\0";

        let devices = unsafe {
            daily_core_context_device_manager_enumerated_devices(
                self.device_manager.as_ptr() as *const _
            )
        };

        if devices.is_null() {
            EMPTY.as_ptr().cast_mut() as *mut _
        } else {
            // NOTE(aleix): Leaking because get_enumerated_devices() uses CStr.
            devices as *mut _
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_user_media(
        &self,
        peer_connection_factory: *mut WebrtcPeerConnectionFactory,
        signaling_thread: *mut WebrtcThread,
        worker_thread: *mut WebrtcThread,
        network_thread: *mut WebrtcThread,
        constraints: *const libc::c_char,
    ) -> *mut libc::c_void {
        let c_str = unsafe { CStr::from_ptr(constraints) };
        tracing::info!("Get user media: {}", c_str.to_str().unwrap());

        unsafe {
            daily_core_context_device_manager_get_user_media(
                self.device_manager.as_ptr() as *mut _,
                peer_connection_factory,
                signaling_thread,
                worker_thread,
                network_thread,
                constraints,
            )
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn create_audio_device_module(
        &self,
        task_queue_factory: *mut WebrtcTaskQueueFactory,
    ) -> *mut WebrtcAudioDeviceModule {
        unsafe {
            daily_core_context_create_audio_device_module(
                self.device_manager.as_ptr() as *mut _,
                task_queue_factory,
            )
        }
    }

    pub fn create_camera_device(
        &self,
        device_name: &str,
        width: u32,
        height: u32,
        color_format: &str,
    ) -> PyResult<PyVirtualCameraDevice> {
        tracing::info!(
            "Creating virtual camera device: {device_name} ({width}x{height} {color_format})"
        );

        let device_name_cstr =
            CString::new(device_name).expect("invalid virtual camera device name string");
        let color_format_cstr = CString::new(color_format).expect("invalid color format string");

        if let Ok(color_format) = ColorFormat::from_str(color_format) {
            let mut py_device =
                PyVirtualCameraDevice::new(device_name, width, height, color_format);

            unsafe {
                let camera_device = daily_core_context_create_virtual_camera_device(
                    self.device_manager.as_ptr() as *mut _,
                    device_name_cstr.as_ptr(),
                    width,
                    height,
                    color_format_cstr.as_ptr(),
                );

                py_device
                    .attach_camera_device(NativeVirtualCameraDevice::from(camera_device as *mut _));
            }

            Ok(py_device)
        } else {
            Err(exceptions::PyValueError::new_err(format!(
                "invalid color format '{color_format}'"
            )))
        }
    }

    pub fn create_speaker_device(
        &self,
        device_name: &str,
        sample_rate: u32,
        channels: u8,
        non_blocking: bool,
    ) -> PyResult<PyVirtualSpeakerDevice> {
        tracing::info!(
            "Creating virtual speaker device: {device_name} ({sample_rate}, {channels} channels, non-blocking: {non_blocking})"
        );

        let device_name_cstr =
            CString::new(device_name).expect("invalid virtual speaker device name string");

        let mut py_device =
            PyVirtualSpeakerDevice::new(device_name, sample_rate, channels, non_blocking);

        unsafe {
            let speaker_device = daily_core_context_create_virtual_speaker_device(
                self.device_manager.as_ptr() as *mut _,
                device_name_cstr.as_ptr(),
                sample_rate,
                channels,
                non_blocking,
            );

            py_device.attach_audio_device(NativeVirtualSpeakerDevice::from(speaker_device));
        }

        Ok(py_device)
    }

    pub fn create_microphone_device(
        &self,
        device_name: &str,
        sample_rate: u32,
        channels: u8,
        non_blocking: bool,
    ) -> PyResult<PyVirtualMicrophoneDevice> {
        tracing::info!(
            "Creating virtual microphone device: {device_name} ({sample_rate}, {channels} channels, non-blocking: {non_blocking})"
        );

        let device_name_cstr =
            CString::new(device_name).expect("invalid virtual microphone device name string");

        let mut py_device = PyVirtualMicrophoneDevice::new(device_name, sample_rate, channels);

        unsafe {
            let microphone_device = daily_core_context_create_virtual_microphone_device(
                self.device_manager.as_ptr() as *mut _,
                device_name_cstr.as_ptr(),
                sample_rate,
                channels,
                non_blocking,
            );

            py_device.attach_audio_device(NativeVirtualMicrophoneDevice::from(microphone_device));
        }

        Ok(py_device)
    }

    pub fn create_native_vad(
        &self,
        reset_period_ms: u32,
        sample_rate: u32,
        channels: u8,
    ) -> PyResult<PyNativeVad> {
        tracing::info!(
            "Creating native VAD ({sample_rate}, {channels} channels, reset period ms: {reset_period_ms})"
        );

        let mut py_vad = PyNativeVad::new(reset_period_ms, sample_rate, channels);

        unsafe {
            let webrtc_vad = daily_core_context_create_vad(reset_period_ms, sample_rate, channels);

            py_vad.attach_webrtc_vad(NativeWebrtcVad::from(webrtc_vad));
        }

        Ok(py_vad)
    }

    pub fn select_speaker_device(&self, device_name: &str) -> PyResult<()> {
        tracing::info!("Selecting virtual speaker device: {device_name})");

        let device_name_cstr =
            CString::new(device_name).expect("invalid virtual speaker device name string");

        let selected = unsafe {
            daily_core_context_select_speaker_device(
                self.device_manager.as_ptr() as *mut _,
                device_name_cstr.as_ptr(),
            )
        };

        if selected {
            Ok(())
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "unable to select virtual speaker device",
            ))
        }
    }

    pub fn get_selected_microphone_device(&self) -> *const libc::c_char {
        const EMPTY: &[u8] = b"\0";

        let device = unsafe {
            daily_core_context_get_selected_microphone_device(
                self.device_manager.as_ptr() as *const _
            )
        };

        if device.is_null() {
            EMPTY.as_ptr().cast()
        } else {
            let microphone = NativeVirtualMicrophoneDevice::from(device);
            microphone.name()
        }
    }
}
