use webrtc_daily::sys::virtual_speaker_device::NativeVirtualSpeakerDevice;

use daily_core::prelude::daily_core_context_virtual_speaker_device_read_frames;

use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// This class represents a virtual speaker device. Virtual speaker devices are
/// used to receive audio from the meeting.
///
/// Virtual speaker devices can be created as blocking or non-blocking (see
/// :func:`Daily.create_speakler_device`). Blocking means that calling
/// :func:`VirtualSpeakerDevice.read_frames` behaves synchronously until all the
/// given audio frames have been read. In contrast, non-blocking will behave
/// asynchronously (i.e. it won't wait).
///
/// The audio format used by virtual speaker devices is 16-bit linear PCM.
#[derive(Clone)]
#[pyclass(name = "VirtualSpeakerDevice", module = "daily")]
pub struct PyVirtualSpeakerDevice {
    device_name: String,
    sample_rate: u32,
    channels: u8,
    audio_device: Option<NativeVirtualSpeakerDevice>,
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

    pub fn attach_audio_device(&mut self, audio_device: NativeVirtualSpeakerDevice) {
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

    /// Reads audio frames from a virtual speaker device created with
    /// :func:`Daily.create_speaker_device`.
    ///
    /// :param int num_frames: The number of audio frames to read
    ///
    /// :return: The read audio frames as a bytestring. If no audio frames could be read, it returns an empty bytestring
    /// :rtype: bytestring.
    pub fn read_frames(&self, py: Python<'_>, num_frames: usize) -> PyResult<PyObject> {
        if let Some(audio_device) = self.audio_device.as_ref() {
            // libwebrtc provides with 16-bit linear PCM
            let num_bytes = num_frames * self.channels() as usize * 2;
            let num_words = num_bytes / 2;

            let mut buffer: Vec<i16> = Vec::with_capacity(num_words);
            let buffer_bytes = buffer.as_mut_slice();

            let frames_read = py.allow_threads(move || unsafe {
                daily_core_context_virtual_speaker_device_read_frames(
                    audio_device.as_ptr() as *mut _,
                    buffer_bytes.as_mut_ptr(),
                    num_frames,
                )
            });

            if frames_read == num_frames as i32 {
                let py_bytes =
                    unsafe { PyBytes::from_ptr(py, buffer.as_ptr() as *const u8, num_bytes) };
                Ok(py_bytes.into_py(py))
            } else if frames_read == 0 {
                let empty_bytes: [u8; 0] = [];
                let py_bytes = PyBytes::new(py, &empty_bytes);
                Ok(py_bytes.into_py(py))
            } else {
                Err(exceptions::PyIOError::new_err(
                    "error reading audio frames from the device",
                ))
            }
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "no speaker device has been attached",
            ))
        }
    }
}
