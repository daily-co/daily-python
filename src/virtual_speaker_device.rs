use webrtc_daily::sys::{
    custom_speaker_device::NativeCustomSpeakerDevice,
    webrtc_daily_custom_speaker_device_read_samples,
};

use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// This class represents a virtual speaker device. Virtual speaker devices are
/// used to receive audio from the meeting.
///
/// The audio format used by virtual speaker devices is 16-bit linear PCM.
#[derive(Clone, Debug)]
#[pyclass(name = "VirtualSpeakerDevice", module = "daily")]
pub struct PyVirtualSpeakerDevice {
    device_name: String,
    sample_rate: u32,
    channels: u8,
    audio_device: Option<NativeCustomSpeakerDevice>,
}

impl PyVirtualSpeakerDevice {
    pub fn new(device_name: &str, sample_rate: u32, channels: u8) -> Self {
        Self {
            device_name: device_name.to_string(),
            sample_rate,
            channels,
            audio_device: None,
        }
    }

    pub fn attach_audio_device(&mut self, audio_device: NativeCustomSpeakerDevice) {
        self.audio_device = Some(audio_device);
    }
}

#[pymethods]
impl PyVirtualSpeakerDevice {
    /// Returns the device name.
    ///
    /// :return: The virtual speaker device name
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

    /// Reads audio samples from a virtual speaker device created with
    /// :func:`Daily.create_speaker_device`.
    ///
    /// The number of audio samples should be multiple of 10ms of audio samples
    /// of the configured sample rate. For example, if the sample rate is 16000
    /// we should be able to read 160 (10ms), 320 (20ms), 480 (30ms), etc.
    ///
    /// :param int num_samples: The number of samples to read
    ///
    /// :return: The read samples as a bytestring. If no samples could be read yet, it returns an empty bytestring
    /// :rtype: bytestring.
    pub fn read_samples(&self, num_samples: usize) -> PyResult<PyObject> {
        if let Some(audio_device) = self.audio_device.as_ref() {
            Python::with_gil(|py| {
                // libwebrtc provides with 16-bit linear PCM
                let bits_per_sample = 16;
                let num_bytes = num_samples * (bits_per_sample * self.channels() as usize) / 8;

                let mut bytes: Vec<u8> = Vec::with_capacity(num_bytes);

                let samples_read = unsafe {
                    webrtc_daily_custom_speaker_device_read_samples(
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
}
