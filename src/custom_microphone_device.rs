use webrtc_daily::sys::{
    custom_microphone_device::NativeCustomMicrophoneDevice,
    webrtc_daily_custom_microphone_device_write_samples,
};

use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// This class represents a custom microphone device. Custom microphone devices
/// are used to send audio to the meeting.
///
/// The audio format used by custom microphone devices is 16-bit linear PCM.
#[derive(Clone, Debug)]
#[pyclass(name = "CustomMicrophoneDevice", module = "daily")]
pub struct PyCustomMicrophoneDevice {
    device_name: String,
    sample_rate: u32,
    channels: u8,
    audio_device: Option<NativeCustomMicrophoneDevice>,
}

impl PyCustomMicrophoneDevice {
    pub fn new(device_name: &str, sample_rate: u32, channels: u8) -> Self {
        Self {
            device_name: device_name.to_string(),
            sample_rate,
            channels,
            audio_device: None,
        }
    }

    pub fn attach_audio_device(&mut self, audio_device: NativeCustomMicrophoneDevice) {
        self.audio_device = Some(audio_device);
    }
}

#[pymethods]
impl PyCustomMicrophoneDevice {
    /// Returns the device name.
    ///
    /// :return: The custom audio device name
    /// :rtype: str
    #[getter]
    fn name(&self) -> String {
        self.device_name.clone()
    }

    /// Returns the sample rate of this device (e.g. 16000).
    ///
    /// :return: The sample rate
    /// :rtype: int
    #[getter]
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Returns the number of channels (2 for stereo and 1 for mono) of this device.
    ///
    /// :return: The number of channels
    /// :rtype: int
    #[getter]
    fn channels(&self) -> u8 {
        self.channels
    }

    /// Writes audio samples to a custom microphone device created with
    /// :func:`Daily.create_microphone_device`.
    ///
    /// The number of audio samples should be multiple of 10ms of audio samples
    /// of the configured sample rate. For example, if the sample rate is 16000
    /// we should be able to read 160 (10ms), 320 (20ms), 480 (30ms), etc.
    ///
    /// :param bytestring num_samples: A bytestring with the samples to write
    /// :param int num_samples: The number of samples to write
    ///
    /// :return: The number of samples written (which should match `num_samples`) or 0 if samples could not still be written
    /// :rtype: int
    pub fn write_samples(&self, samples: PyObject, num_samples: usize) -> PyResult<PyObject> {
        if let Some(audio_device) = self.audio_device.as_ref() {
            Python::with_gil(|py| {
                let py_samples: &PyBytes = samples.downcast::<PyBytes>(py).unwrap();

                let samples_written = unsafe {
                    webrtc_daily_custom_microphone_device_write_samples(
                        audio_device.as_ptr() as *mut _,
                        py_samples.as_bytes().as_ptr() as *const _,
                        num_samples,
                    )
                };

                if samples_written == num_samples as i32 || samples_written == 0 {
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
