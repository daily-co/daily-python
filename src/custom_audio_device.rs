use webrtc_daily::sys::{
    custom_audio_device::NativeCustomAudioDevice, webrtc_daily_custom_audio_device_read_samples,
    webrtc_daily_custom_audio_device_write_samples,
};

use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[derive(Clone, Debug)]
#[pyclass(name = "CustomAudioDevice", module = "daily")]
pub struct PyCustomAudioDevice {
    device_name: String,
    play_sample_rate: u32,
    play_channels: u8,
    rec_sample_rate: u32,
    rec_channels: u8,
    audio_device: Option<NativeCustomAudioDevice>,
}

impl PyCustomAudioDevice {
    pub fn new(
        device_name: &str,
        play_sample_rate: u32,
        play_channels: u8,
        rec_sample_rate: u32,
        rec_channels: u8,
    ) -> Self {
        Self {
            device_name: device_name.to_string(),
            play_sample_rate,
            play_channels,
            rec_sample_rate,
            rec_channels,
            audio_device: None,
        }
    }

    pub fn attach_audio_device(&mut self, audio_device: NativeCustomAudioDevice) {
        self.audio_device = Some(audio_device);
    }
}

#[pymethods]
impl PyCustomAudioDevice {
    pub fn name(&self) -> String {
        self.device_name.clone()
    }

    pub fn play_sample_rate(&self) -> u32 {
        self.play_sample_rate
    }

    pub fn play_channels(&self) -> u8 {
        self.play_channels
    }

    pub fn recording_sample_rate(&self) -> u32 {
        self.rec_sample_rate
    }

    pub fn recording_channels(&self) -> u8 {
        self.rec_channels
    }

    pub fn read_samples(&self, num_samples: usize) -> PyResult<PyObject> {
        if let Some(audio_device) = self.audio_device.as_ref() {
            Python::with_gil(|py| {
                // libwebrtc provides with 16-bit linear PCM
                let bits_per_sample = 16;
                let num_bytes = num_samples * (bits_per_sample * self.play_channels() as usize) / 8;

                let mut bytes: Vec<u8> = Vec::with_capacity(num_bytes);

                let samples_read = unsafe {
                    webrtc_daily_custom_audio_device_read_samples(
                        audio_device.as_ptr() as *mut _,
                        bytes.as_mut_ptr() as *mut _,
                        num_samples,
                    )
                };

                if samples_read == num_samples as i32 {
                    let py_bytes = unsafe { PyBytes::from_ptr(py, bytes.as_ptr(), num_bytes) };
                    Ok(py_bytes.into_py(py))
                } else if samples_read == 0 {
                    let empty_bytes: [u8; 0] = [];
                    let py_bytes = PyBytes::new(py, &empty_bytes);
                    Ok(py_bytes.into_py(py))
                } else {
                    Err(exceptions::PyIOError::new_err(
                        "error writing audio samples to device",
                    ))
                }
            })
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "no device has been attached",
            ))
        }
    }

    pub fn write_samples(&self, py_samples: PyObject, num_samples: usize) -> PyResult<PyObject> {
        if let Some(audio_device) = self.audio_device.as_ref() {
            Python::with_gil(|py| {
                let samples: &PyBytes = py_samples.downcast::<PyBytes>(py).unwrap();

                let samples_written = unsafe {
                    webrtc_daily_custom_audio_device_write_samples(
                        audio_device.as_ptr() as *mut _,
                        samples.as_bytes().as_ptr() as *const _,
                        num_samples,
                    )
                };

                if samples_written == num_samples as i32 {
                    Ok(samples_written.into_py(py))
                } else if samples_written == 0 {
                    Ok(samples_written.into_py(py))
                } else {
                    Err(exceptions::PyIOError::new_err(
                        "error writing audio samples to device",
                    ))
                }
            })
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "no device has been attached",
            ))
        }
    }
}
