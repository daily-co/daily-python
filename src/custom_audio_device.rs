use webrtc_daily::sys::{
    custom_audio_device::NativeCustomAudioDevice, webrtc_daily_custom_audio_device_read_samples,
    webrtc_daily_custom_audio_device_write_samples,
};

use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// This class represents a custom audio device. Custom audio devices are used
/// to receive or to send audio. Custom audio devices behave like system
/// speakers or microphone therefore, for example, it is not possible to receive
/// audio for a specific participant.
///
/// The audio format used by custom audio devices (both for sending and receving)
/// is 16-bit linear PCM.
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
    /// Returns the device name.
    ///
    /// :return: The custom audio device name
    /// :rtype: str
    #[getter]
    fn name(&self) -> String {
        self.device_name.clone()
    }

    /// Returns the play out sample rate of this device (e.g. 16000).
    ///
    /// :return: The play out sample rate
    /// :rtype: int
    #[getter]
    fn play_sample_rate(&self) -> u32 {
        self.play_sample_rate
    }

    /// Returns the play out number of channels (2 for stereo and 1 for mono).
    ///
    /// :return: The play out number of channels
    /// :rtype: int
    #[getter]
    fn play_channels(&self) -> u8 {
        self.play_channels
    }

    /// Returns the recording sample rate of this device (e.g. 16000).
    ///
    /// :return: The recording sample rate
    /// :rtype: int
    #[getter]
    fn recording_sample_rate(&self) -> u32 {
        self.rec_sample_rate
    }

    /// Returns the recording number of channels (2 for stereo and 1 for mono).
    ///
    /// :return: The recording number of channels
    /// :rtype: int
    #[getter]
    fn recording_channels(&self) -> u8 {
        self.rec_channels
    }

    /// Reads audio samples from a custom audio device created with
    /// :func:`Daily.create_custom_audio_device`.
    ///
    /// The number of audio samples should be multiple of 10ms of audio samples
    /// of the configured play sample rate. For example, if the play sample rate
    /// is 16000 we should be able to read 160 (10ms), 320 (20ms), 480 (30ms),
    /// etc.
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

    /// Writes audio samples to a custom audio device created with
    /// :func:`Daily.create_custom_audio_device`.
    ///
    /// The number of audio samples should be multiple of 10ms of audio samples
    /// of the configured recording sample rate. For example, if the recording
    /// sample rate is 16000 we should be able to read 160 (10ms), 320 (20ms),
    /// 480 (30ms), etc.
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
                    webrtc_daily_custom_audio_device_write_samples(
                        audio_device.as_ptr() as *mut _,
                        py_samples.as_bytes().as_ptr() as *const _,
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
