use std::ffi::CString;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::PyVirtualMicrophoneDevice;
use crate::PyVirtualSpeakerDevice;

use webrtc_daily::sys::{
    audio_device_module::NativeAudioDeviceModule, media_stream::MediaStream,
    rtc_refcount_interface_addref, virtual_microphone_device::NativeVirtualMicrophoneDevice,
    virtual_speaker_device::NativeVirtualSpeakerDevice,
};

use daily_core::prelude::{
    daily_core_context_create_audio_device_module,
    daily_core_context_create_virtual_microphone_device,
    daily_core_context_create_virtual_speaker_device, daily_core_context_custom_get_user_media,
    daily_core_context_get_selected_virtual_microphone_device,
    daily_core_context_select_virtual_microphone_device,
    daily_core_context_select_virtual_speaker_device, WebrtcAudioDeviceModule,
    WebrtcPeerConnectionFactory, WebrtcTaskQueueFactory, WebrtcThread,
};

use pyo3::exceptions;
use pyo3::prelude::*;

// This should be initialized from Daily.init().
pub static mut GLOBAL_CONTEXT: Option<DailyContext> = None;

#[derive(Default)]
pub struct DailyContext {
    request_id: AtomicU64,
    audio_device_module: Option<NativeAudioDeviceModule>,
}

impl DailyContext {
    pub fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn get_enumerated_devices(&self) -> *mut libc::c_char {
        const EMPTY: &[u8] = b"[]\0";

        if let Some(adm) = self.audio_device_module.as_ref() {
            let devices = unsafe {
                webrtc_daily::sys::webrtc_daily_custom_audio_enumerated_devices(
                    adm.as_ptr() as *mut _
                )
            };

            if devices.is_null() {
                EMPTY.as_ptr().cast_mut() as *mut _
            } else {
                // NOTE(aleix): Leaking because get_enumerated_devices() uses CStr.
                devices as *mut _
            }
        } else {
            EMPTY.as_ptr().cast_mut() as *mut _
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
        if let Some(adm) = self.audio_device_module.as_mut() {
            unsafe {
                daily_core_context_custom_get_user_media(
                    adm.as_mut_ptr() as *mut _,
                    peer_connection_factory,
                    signaling_thread,
                    worker_thread,
                    network_thread,
                    constraints,
                )
            }
        } else if let Ok(mut media_stream) = MediaStream::new() {
            // Increase the reference count because it's decremented on drop
            // and we want to return a valid pointer.
            unsafe {
                rtc_refcount_interface_addref(media_stream.as_mut_ptr());
            }

            media_stream.as_mut_ptr() as *mut _
        } else {
            ptr::null_mut()
        }
    }

    pub fn create_audio_device_module(
        &mut self,
        task_queue_factory: *mut WebrtcTaskQueueFactory,
    ) -> *mut WebrtcAudioDeviceModule {
        unsafe {
            let adm = daily_core_context_create_audio_device_module(task_queue_factory);

            self.audio_device_module =
                Some(NativeAudioDeviceModule::from_unretained(adm as *mut _));

            adm
        }
    }

    pub fn create_speaker_device(
        &mut self,
        device_name: &str,
        sample_rate: u32,
        channels: u8,
    ) -> PyResult<PyVirtualSpeakerDevice> {
        if let Some(adm) = self.audio_device_module.as_mut() {
            let device_name_cstr =
                CString::new(device_name).expect("invalid virtual speaker device name string");

            let mut py_device = PyVirtualSpeakerDevice::new(device_name, sample_rate, channels);

            unsafe {
                let speaker_device = daily_core_context_create_virtual_speaker_device(
                    adm.as_mut_ptr() as *mut _,
                    device_name_cstr.as_ptr(),
                    sample_rate,
                    channels,
                );

                py_device.attach_audio_device(NativeVirtualSpeakerDevice::from_unretained(
                    speaker_device as *mut _,
                ));
            }

            Ok(py_device)
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "did you call Daily.init(virtual_devices = True)?",
            ))
        }
    }

    pub fn create_microphone_device(
        &mut self,
        device_name: &str,
        sample_rate: u32,
        channels: u8,
    ) -> PyResult<PyVirtualMicrophoneDevice> {
        if let Some(adm) = self.audio_device_module.as_mut() {
            let device_name_cstr =
                CString::new(device_name).expect("invalid virtual microphone device name string");

            let mut py_device = PyVirtualMicrophoneDevice::new(device_name, sample_rate, channels);

            unsafe {
                let microphone_device = daily_core_context_create_virtual_microphone_device(
                    adm.as_mut_ptr() as *mut _,
                    device_name_cstr.as_ptr(),
                    sample_rate,
                    channels,
                );

                py_device.attach_audio_device(NativeVirtualMicrophoneDevice::from_unretained(
                    microphone_device as *mut _,
                ));
            }

            Ok(py_device)
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "did you call Daily.init(virtual_devices = True)?",
            ))
        }
    }

    pub fn select_speaker_device(&mut self, device_name: &str) -> PyResult<()> {
        if let Some(adm) = self.audio_device_module.as_ref() {
            let device_name_cstr =
                CString::new(device_name).expect("invalid virtual speaker device name string");

            let selected = unsafe {
                daily_core_context_select_virtual_speaker_device(
                    adm.as_ptr() as *mut _,
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
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "did you call Daily.init(virtual_devices = True)?",
            ))
        }
    }

    pub fn select_microphone_device(&mut self, device_name: &str) -> PyResult<()> {
        if let Some(adm) = self.audio_device_module.as_ref() {
            let device_name_cstr =
                CString::new(device_name).expect("invalid virtual microphone device name string");

            let selected = unsafe {
                daily_core_context_select_virtual_microphone_device(
                    adm.as_ptr() as *mut _,
                    device_name_cstr.as_ptr(),
                )
            };

            if selected {
                Ok(())
            } else {
                Err(exceptions::PyRuntimeError::new_err(
                    "unable to select virtual microphone device",
                ))
            }
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "did you call Daily.init(virtual_devices = True)?",
            ))
        }
    }

    pub fn get_selected_microphone_device(&self) -> *const libc::c_char {
        const EMPTY: &[u8] = b"\0";

        if let Some(adm) = self.audio_device_module.as_ref() {
            let device =
                daily_core_context_get_selected_virtual_microphone_device(adm.as_ptr() as *mut _);
            if device.is_null() {
                EMPTY.as_ptr().cast()
            } else {
                // NOTE(aleix): Leaking because get_audio_device() uses CStr.
                device
            }
        } else {
            EMPTY.as_ptr().cast()
        }
    }
}
