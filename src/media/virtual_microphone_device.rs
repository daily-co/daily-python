use std::sync::atomic::{AtomicU64, Ordering};
use std::{collections::HashMap, sync::Mutex};

use crate::util::memory::AlignedI16Data;

use webrtc_daily::sys::virtual_microphone_device::NativeVirtualMicrophoneDevice;

use daily_core::prelude::daily_core_context_virtual_microphone_device_write_frames;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};
use pyo3::{exceptions, IntoPyObjectExt};

/// This class represents a virtual microphone device. Virtual microphone
/// devices are used to send audio to the meeting. Then can be created as
/// blocking or non-blocking (see :func:`Daily.create_microphone_device`). A
/// blocking device will wait until :func:`VirtualMicrophoneDevice.write_frames`
/// finishes writing the given audio frames. In contrast, a non-blocking
/// microphone will not wait.
///
/// NOTE: Virtual microphone devices emulate a hardware device and have the
/// constraint that only one microphone can be active per process. You can
/// select the active microphone through the input settings in
/// :func:`CallClient.join` or :func:`CallClient.update_inputs`. However, it is
/// possible to use a custom microphone audio track when also specifying the
/// input settings.
///
/// The audio format used by virtual microphone devices is 16-bit linear PCM.
#[pyclass(name = "VirtualMicrophoneDevice", module = "daily")]
pub struct PyVirtualMicrophoneDevice {
    device_name: String,
    sample_rate: u32,
    channels: u8,
    audio_device: Option<NativeVirtualMicrophoneDevice>,
    request_id: AtomicU64,
    completions: Mutex<HashMap<u64, Py<PyAny>>>,
}

impl PyVirtualMicrophoneDevice {
    pub fn new(device_name: &str, sample_rate: u32, channels: u8) -> Self {
        Self {
            device_name: device_name.to_string(),
            sample_rate,
            channels,
            audio_device: None,
            request_id: AtomicU64::new(0),
            completions: Mutex::new(HashMap::new()),
        }
    }

    pub fn attach_audio_device(&mut self, audio_device: NativeVirtualMicrophoneDevice) {
        self.audio_device = Some(audio_device);
    }

    fn maybe_register_completion(&mut self, completion: Option<Py<PyAny>>) -> u64 {
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
impl PyVirtualMicrophoneDevice {
    /// Returns the device name.
    ///
    /// :return: The virtual microphone device name
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

    /// Writes audio frames to a virtual microphone device created with
    /// :func:`Daily.create_microphone_device`. For non-blocking devices, the
    /// completion callback will be called when the audio frames have been
    /// written.
    ///
    /// If less than a multiple of 10ms worth of audio frames are provided
    /// on a blocking microphone, padding will be added up to the next multiple.
    ///
    /// :param bytestring frames: A bytestring with the audio frames to write
    /// :param func completion: An optional completion callback with one parameter: (int)
    ///
    /// :return: The number of audio frames written
    /// :rtype: int
    #[pyo3(signature = (frames, completion = None))]
    pub fn write_frames(
        &mut self,
        frames: &Bound<'_, PyBytes>,
        completion: Option<Py<PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        if self.audio_device.is_none() {
            return Err(exceptions::PyRuntimeError::new_err(
                "no microphone device has been attached",
            ));
        }

        let num_bytes = frames.len()?;

        let bytes_per_sample: usize = 2;

        // libwebrtc needs 16-bit linear PCM samples
        if num_bytes % (bytes_per_sample * self.channels as usize) != 0 {
            return Err(exceptions::PyValueError::new_err(
                "frames bytestring should contain 16-bit samples",
            ));
        }

        let num_frames = (num_bytes / bytes_per_sample) / self.channels as usize;

        let bytes = frames.as_bytes();
        let aligned = AlignedI16Data::new(bytes);

        let request_id = self.maybe_register_completion(completion);

        tracing::trace!(
            "Writing audio frames to {} ({num_frames} frames, request {request_id})",
            self.device_name
        );

        Python::attach(|py| {
            let frames_written = py.detach(move || unsafe {
                daily_core_context_virtual_microphone_device_write_frames(
                    self.audio_device.as_ref().unwrap().as_ptr() as *mut _,
                    aligned.as_ptr(),
                    num_frames,
                    request_id,
                    on_write_frames,
                    self as *const PyVirtualMicrophoneDevice as *mut libc::c_void,
                )
            });

            if frames_written >= 0 {
                Ok(frames_written.into_py_any(py).unwrap())
            } else {
                Err(exceptions::PyIOError::new_err(
                    "error writing audio frames to device",
                ))
            }
        })
    }
}

pub(crate) unsafe extern "C" fn on_write_frames(
    device: *mut libc::c_void,
    request_id: u64,
    num_frames: usize,
) {
    let microphone: &mut PyVirtualMicrophoneDevice =
        unsafe { &mut *(device as *mut PyVirtualMicrophoneDevice) };

    Python::attach(|py| {
        let completion = microphone.completions.lock().unwrap().remove(&request_id);

        if let Some(completion) = completion {
            let args = PyTuple::new(py, &[num_frames.into_py_any(py).unwrap()]).unwrap();

            tracing::trace!(
                "Finished writing audio frames to {} ({num_frames} frames, request {request_id})",
                microphone.device_name
            );

            if let Err(error) = completion.call1(py, args) {
                error.write_unraisable(py, None);
            }
        }
    })
}
