use std::sync::atomic::{AtomicU64, Ordering};
use std::{collections::HashMap, sync::Mutex};

use crate::util::memory::AlignedI16Data;

use daily_core::prelude::*;

use webrtc_daily::sys::custom_audio_source::NativeDailyAudioSource;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};
use pyo3::{exceptions, IntoPyObjectExt};

/// This class represents a custom audio source. Custom audio sources are used
/// to send audio to an audio track. See
/// :func:`daily.CallClient.add_custom_audio_track`.
///
/// The audio format used by custom audio sources is 16-bit linear PCM.
#[pyclass(name = "CustomAudioSource", module = "daily")]
pub struct PyCustomAudioSource {
    pub sample_rate: u32,
    pub channels: u8,
    pub audio_source: NativeDailyAudioSource,
    request_id: AtomicU64,
    completions: Mutex<HashMap<u64, PyObject>>,
}

impl PyCustomAudioSource {
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
impl PyCustomAudioSource {
    #[new]
    pub fn new(sample_rate: u32, channels: u8) -> Self {
        let audio_source_ptr = unsafe {
            daily_core_context_create_custom_audio_source_with_silence(
                sample_rate as i32,
                channels as usize,
            )
        };

        let audio_source = NativeDailyAudioSource::from(audio_source_ptr);

        Self {
            sample_rate,
            channels,
            audio_source,
            request_id: AtomicU64::new(0),
            completions: Mutex::new(HashMap::new()),
        }
    }

    /// Returns the sample rate of this audio source (e.g. 16000).
    ///
    /// :return: The sample rate
    /// :rtype: int
    #[getter]
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Returns the number of channels (2 for stereo and 1 for mono) of this
    /// audio source.
    ///
    /// :return: The number of channels
    /// :rtype: int
    #[getter]
    fn channels(&self) -> u8 {
        self.channels
    }

    /// Writes audio frames to the audio source. The frames will be sent to the
    /// audio track if this source is attached to a track.
    ///
    /// This function blocks, if a `completion` callback is not provided, until
    /// the audio frames have been written. If a `completion` callback is
    /// provided this function is non-blocking and the `completion` callback
    /// will be called when the audio frames are written.
    ///
    /// :param bytestring frames: A bytestring with the audio frames to write
    /// :param func completion: An optional completion callback with one parameter: (int)
    ///
    /// :return: The number of audio frames written
    /// :rtype: int
    #[pyo3(signature = (frames, completion=None))]
    pub fn write_frames(
        &mut self,
        frames: &Bound<'_, PyBytes>,
        completion: Option<PyObject>,
    ) -> PyResult<PyObject> {
        let num_bytes = frames.len()?;
        let bytes_per_sample: usize = 2;

        if num_bytes % (bytes_per_sample * self.channels as usize) != 0 {
            return Err(exceptions::PyValueError::new_err(
                "frames bytestring should contain 16-bit samples",
            ));
        }

        let num_frames = (num_bytes / bytes_per_sample) / self.channels as usize;

        let bytes = frames.as_bytes();
        let aligned = AlignedI16Data::new(bytes);

        let request_id = self.maybe_register_completion(completion.clone());

        tracing::trace!(
            "Writing audio frames to audio source {:?} ({num_frames} frames, request {request_id})",
            self.audio_source.as_ptr()
        );

        Python::with_gil(|py| {
            let frames_written = py.allow_threads(move || unsafe {
                if completion.is_none() {
                    daily_core_context_custom_audio_source_write_frames_sync(
                        self.audio_source.as_ptr() as *mut _,
                        aligned.as_ptr() as *const _,
                        (bytes_per_sample * 8) as i32,
                        self.sample_rate as i32,
                        self.channels as usize,
                        num_frames,
                    )
                } else {
                    daily_core_context_custom_audio_source_write_frames_async(
                        self.audio_source.as_ptr() as *mut _,
                        aligned.as_ptr() as *const _,
                        (bytes_per_sample * 8) as i32,
                        self.sample_rate as i32,
                        self.channels as usize,
                        num_frames,
                        request_id,
                        on_write_frames,
                        self as *const PyCustomAudioSource as *mut libc::c_void,
                    )
                }
            });

            if frames_written >= 0 {
                Ok(frames_written.into_py_any(py).unwrap())
            } else {
                Err(exceptions::PyIOError::new_err(
                    "error writing audio frames to audio source",
                ))
            }
        })
    }
}

pub(crate) unsafe extern "C" fn on_write_frames(
    source: *mut libc::c_void,
    request_id: u64,
    num_frames: usize,
) {
    let audio_source: &mut PyCustomAudioSource =
        unsafe { &mut *(source as *mut PyCustomAudioSource) };

    Python::with_gil(|py| {
        let completion = audio_source.completions.lock().unwrap().remove(&request_id);

        let args = PyTuple::new(py, &[num_frames.into_py_any(py).unwrap()]).unwrap();

        tracing::trace!(
            "Finished writing audio frames to audio source {:?} ({num_frames} frames, request {request_id})",
            audio_source.audio_source.as_ptr()
        );

        if let Some(completion) = completion {
            if let Err(error) = completion.call1(py, args) {
                error.write_unraisable(py, None);
            }
        }
    })
}
