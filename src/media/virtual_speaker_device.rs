use std::sync::atomic::{AtomicU64, Ordering};
use std::{collections::HashMap, sync::Mutex};

use webrtc_daily::sys::virtual_speaker_device::NativeVirtualSpeakerDevice;

use daily_core::prelude::daily_core_context_virtual_speaker_device_read_frames;

use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};

/// This class represents a virtual speaker device. Virtual speaker devices are
/// used to receive audio from the meeting. They can be created as blocking or
/// non-blocking (see :func:`Daily.create_speakler_device`). Blocking means that
/// calling :func:`VirtualSpeakerDevice.read_frames` behaves synchronously until
/// all the given audio frames have been read. In contrast, non-blocking will
/// behave asynchronously (i.e. it won't wait).
///
/// NOTE: Virtual speaker devices emulate a hardware device and have the
/// constraint that only one speaker can be active per process. You can select
/// the active speaker with :func:`Daily.select_speaker_device`.
///
/// The audio format used by virtual speaker devices is 16-bit linear PCM.
#[pyclass(name = "VirtualSpeakerDevice", module = "daily")]
pub struct PyVirtualSpeakerDevice {
    device_name: String,
    sample_rate: u32,
    channels: u8,
    non_blocking: bool,
    audio_device: Option<NativeVirtualSpeakerDevice>,
    request_id: AtomicU64,
    completions: Mutex<HashMap<u64, PyObject>>,
}

impl PyVirtualSpeakerDevice {
    pub fn new(device_name: &str, sample_rate: u32, channels: u8, non_blocking: bool) -> Self {
        Self {
            device_name: device_name.to_string(),
            sample_rate,
            channels,
            non_blocking,
            audio_device: None,
            request_id: AtomicU64::new(0),
            completions: Mutex::new(HashMap::new()),
        }
    }

    pub fn attach_audio_device(&mut self, audio_device: NativeVirtualSpeakerDevice) {
        self.audio_device = Some(audio_device);
    }

    fn maybe_register_completion(&mut self, completion: Option<PyObject>) -> u64 {
        let request_id = self.request_id.fetch_add(1, Ordering::SeqCst);

        if let Some(completion) = completion {
            self.completions
                .lock()
                .unwrap()
                .insert(request_id, completion);
        }

        request_id
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
    /// :func:`Daily.create_speaker_device`. For non-blocking devices, the
    /// completion callback will be called when the audio frames have been read.
    ///
    /// :param int num_frames: The number of audio frames to read
    /// :param func completion: An optional completion callback with one parameter: (bytestring)
    ///
    /// :return: The read audio frames as a bytestring, or an empty bytestring if no frames were read
    /// :rtype: bytestring.
    #[pyo3(signature = (num_frames, completion = None))]
    pub fn read_frames(
        &mut self,
        num_frames: usize,
        completion: Option<PyObject>,
    ) -> PyResult<PyObject> {
        if self.audio_device.is_none() {
            return Err(exceptions::PyRuntimeError::new_err(
                "no speaker device has been attached",
            ));
        }

        // In the non-blocking case, we don't want to allocate memory here
        // since we will exit the function right away and the memory won't
        // be valid. The needed memory will be allocated internally.
        let num_bytes = if self.non_blocking {
            0
        } else {
            // libwebrtc provides with 16-bit linear PCM
            let bytes_per_sample = 2;
            num_frames * self.channels() as usize * bytes_per_sample
        };

        let num_words = num_bytes / 2;

        let mut buffer: Vec<i16> = Vec::with_capacity(num_words);

        let request_id = self.maybe_register_completion(completion);

        let device_name = self.device_name.clone();

        tracing::trace!(
            "Reading audio frames from {device_name} ({num_bytes} bytes, request {request_id})",
        );

        Python::with_gil(move |py| {
            let buffer_bytes = buffer.as_mut_slice();

            let frames_read = py.allow_threads(move || unsafe {
                daily_core_context_virtual_speaker_device_read_frames(
                    self.audio_device.as_ref().unwrap().as_ptr() as *mut _,
                    buffer_bytes.as_mut_ptr(),
                    num_frames,
                    request_id,
                    on_read_frames,
                    self as *const PyVirtualSpeakerDevice as *mut libc::c_void,
                )
            });

            if frames_read == num_frames as i32 {
                let py_bytes =
                    unsafe { PyBytes::from_ptr(py, buffer.as_ptr() as *const u8, num_bytes) };

                tracing::trace!(
                    "Finished reading audio frames from {device_name} ({num_bytes} bytes, request {request_id})"
                );

                Ok(py_bytes.into_any().unbind())
            } else if frames_read == 0 {
                let empty_bytes: [u8; 0] = [];
                let py_bytes = PyBytes::new(py, &empty_bytes);
                Ok(py_bytes.into_any().unbind())
            } else {
                Err(exceptions::PyIOError::new_err(
                    "error reading audio frames from the device",
                ))
            }
        })
    }
}

pub(crate) unsafe extern "C" fn on_read_frames(
    device: *mut libc::c_void,
    request_id: u64,
    frames: *mut i16,
    num_frames: usize,
) {
    let speaker: &mut PyVirtualSpeakerDevice =
        unsafe { &mut *(device as *mut PyVirtualSpeakerDevice) };

    Python::with_gil(|py| {
        let completion = speaker.completions.lock().unwrap().remove(&request_id);

        if let Some(completion) = completion {
            let bytes_per_sample = 2;
            let num_bytes = num_frames * speaker.channels() as usize * bytes_per_sample;
            let empty_bytes: [u8; 0] = [];

            let py_bytes = if num_bytes > 0 {
                unsafe { PyBytes::from_ptr(py, frames as *const u8, num_bytes) }
            } else {
                PyBytes::new(py, &empty_bytes)
            };

            tracing::trace!(
                "Finished reading audio frames from {} ({num_bytes} bytes, request {request_id})",
                speaker.device_name
            );

            let args = PyTuple::new(py, [py_bytes]).unwrap();

            if let Err(error) = completion.call1(py, args) {
                error.write_unraisable(py, None);
            }
        };
    })
}
