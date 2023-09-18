use std::ffi::CString;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::PyVirtualCameraDevice;
use crate::PyVirtualMicrophoneDevice;
use crate::PyVirtualSpeakerDevice;

use webrtc_daily::sys::{
    color_format::ColorFormat, device_manager::NativeDeviceManager,
    virtual_camera_device::NativeVirtualCameraDevice,
    virtual_microphone_device::NativeVirtualMicrophoneDevice,
    virtual_speaker_device::NativeVirtualSpeakerDevice,
};

use daily_core::prelude::{
    daily_core_context_create_audio_device_module, daily_core_context_create_device_manager,
    daily_core_context_create_virtual_camera_device,
    daily_core_context_create_virtual_microphone_device,
    daily_core_context_create_virtual_speaker_device,
    daily_core_context_device_manager_get_user_media,
    daily_core_context_get_selected_microphone_device, daily_core_context_select_speaker_device,
    WebrtcAudioDeviceModule, WebrtcPeerConnectionFactory, WebrtcTaskQueueFactory, WebrtcThread,
};

use pyo3::exceptions;
use pyo3::prelude::*;

// This should be initialized from Daily.init().
pub static mut GLOBAL_CONTEXT: Option<DailyContext> = None;

pub struct DailyContext {
    request_id: AtomicU64,
    device_manager: NativeDeviceManager,
}

impl DailyContext {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let device_manager = unsafe { daily_core_context_create_device_manager() };

        Self {
            device_manager: unsafe {
                NativeDeviceManager::from_unretained(device_manager as *mut _)
            },
            request_id: AtomicU64::new(0),
        }
    }

    pub fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn get_enumerated_devices(&self) -> *mut libc::c_char {
        const EMPTY: &[u8] = b"[]\0";

        let devices = unsafe {
            webrtc_daily::sys::webrtc_daily_device_manager_enumerated_devices(
                self.device_manager.as_ptr(),
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
        &mut self,
        peer_connection_factory: *mut WebrtcPeerConnectionFactory,
        signaling_thread: *mut WebrtcThread,
        worker_thread: *mut WebrtcThread,
        network_thread: *mut WebrtcThread,
        constraints: *const libc::c_char,
    ) -> *mut libc::c_void {
        unsafe {
            daily_core_context_device_manager_get_user_media(
                self.device_manager.as_mut_ptr() as *mut _,
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
        &mut self,
        task_queue_factory: *mut WebrtcTaskQueueFactory,
    ) -> *mut WebrtcAudioDeviceModule {
        unsafe {
            daily_core_context_create_audio_device_module(
                self.device_manager.as_mut_ptr() as *mut _,
                task_queue_factory,
            )
        }
    }

    pub fn create_camera_device(
        &mut self,
        device_name: &str,
        width: u32,
        height: u32,
        color_format: &str,
    ) -> PyResult<PyVirtualCameraDevice> {
        let device_name_cstr =
            CString::new(device_name).expect("invalid virtual camera device name string");
        let color_format_cstr = CString::new(color_format).expect("invalid color format string");

        if let Ok(color_format) = ColorFormat::from_str(color_format) {
            let mut py_device =
                PyVirtualCameraDevice::new(device_name, width, height, color_format);

            unsafe {
                let camera_device = daily_core_context_create_virtual_camera_device(
                    self.device_manager.as_mut_ptr() as *mut _,
                    device_name_cstr.as_ptr(),
                    width,
                    height,
                    color_format_cstr.as_ptr(),
                );

                py_device.attach_camera_device(NativeVirtualCameraDevice::from_unretained(
                    camera_device as *mut _,
                ));
            }

            Ok(py_device)
        } else {
            Err(exceptions::PyValueError::new_err(format!(
                "invalid color format '{color_format}'"
            )))
        }
    }

    pub fn create_speaker_device(
        &mut self,
        device_name: &str,
        sample_rate: u32,
        channels: u8,
    ) -> PyResult<PyVirtualSpeakerDevice> {
        let device_name_cstr =
            CString::new(device_name).expect("invalid virtual speaker device name string");

        let mut py_device = PyVirtualSpeakerDevice::new(device_name, sample_rate, channels);

        unsafe {
            let speaker_device = daily_core_context_create_virtual_speaker_device(
                self.device_manager.as_mut_ptr() as *mut _,
                device_name_cstr.as_ptr(),
                sample_rate,
                channels,
            );

            py_device.attach_audio_device(NativeVirtualSpeakerDevice::from_unretained(
                speaker_device as *mut _,
            ));
        }

        Ok(py_device)
    }

    pub fn create_microphone_device(
        &mut self,
        device_name: &str,
        sample_rate: u32,
        channels: u8,
    ) -> PyResult<PyVirtualMicrophoneDevice> {
        let device_name_cstr =
            CString::new(device_name).expect("invalid virtual microphone device name string");

        let mut py_device = PyVirtualMicrophoneDevice::new(device_name, sample_rate, channels);

        unsafe {
            let microphone_device = daily_core_context_create_virtual_microphone_device(
                self.device_manager.as_mut_ptr() as *mut _,
                device_name_cstr.as_ptr(),
                sample_rate,
                channels,
            );

            py_device.attach_audio_device(NativeVirtualMicrophoneDevice::from_unretained(
                microphone_device as *mut _,
            ));
        }

        Ok(py_device)
    }

    pub fn select_speaker_device(&mut self, device_name: &str) -> PyResult<()> {
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
            // NOTE(aleix): Leaking because get_audio_device() uses CStr.
            device
        }
    }
}
