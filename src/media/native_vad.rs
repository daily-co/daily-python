use crate::util::memory::AlignedI16Data;

use webrtc_daily::sys::vad::NativeWebrtcVad;

use daily_core::prelude::daily_core_context_vad_analyze;

use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// This class represents a Voice Activity Detection (VAD) analyzer. VADs are
/// used to detect speech on an audio stream.
///
/// This VAD implementation works by analyzing 10ms audio frames at a time
/// returning a confidence probability. It is possible to build a more
/// sophisticated VAD (e.g. one that detects long sentences) on top of this one.
///
/// The audio format used by this VAD is 16-bit linear PCM.
#[pyclass(name = "NativeVad", module = "daily")]
pub struct PyNativeVad {
    reset_period_ms: u32,
    sample_rate: u32,
    channels: u8,
    webrtc_vad: Option<NativeWebrtcVad>,
}

impl PyNativeVad {
    pub fn new(reset_period_ms: u32, sample_rate: u32, channels: u8) -> Self {
        Self {
            reset_period_ms,
            sample_rate,
            channels,
            webrtc_vad: None,
        }
    }

    pub fn attach_webrtc_vad(&mut self, webrtc_vad: NativeWebrtcVad) {
        self.webrtc_vad = Some(webrtc_vad);
    }
}

#[pymethods]
impl PyNativeVad {
    /// Returns the number of milliseconds after which the internal VAD is
    /// reset. It should be at least 20ms.
    ///
    /// :return: The sample rate
    /// :rtype: int
    #[getter]
    fn reset_period_ms(&self) -> u32 {
        self.reset_period_ms
    }

    /// Returns the sample rate of incoming audio frames for this VAD
    /// (e.g. 16000).
    ///
    /// :return: The sample rate
    /// :rtype: int
    #[getter]
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Returns the number of channels (2 for stereo and 1 for mono) of incoming
    /// audio frames for this VAD.
    ///
    /// :return: The number of channels
    /// :rtype: int
    #[getter]
    fn channels(&self) -> u8 {
        self.channels
    }

    /// Analyzes 10ms of audio frames and returns the confidence probability
    /// that speech was detected. If more than 10ms of audio frames are given,
    /// only the first 10ms will be used.
    ///
    /// :return: The probability (from 0 to 1.0) that speech was detected
    /// :rtype: float
    fn analyze_frames(&self, frames: &Bound<'_, PyBytes>) -> PyResult<f32> {
        let num_bytes = frames.len()?;
        let bytes_per_sample = 2;

        // libwebrtc needs 16-bit linear PCM samples
        if num_bytes % bytes_per_sample != 0 {
            return Err(exceptions::PyValueError::new_err(
                "frames bytestring should contain 16-bit samples",
            ));
        }

        let num_frames = (num_bytes / bytes_per_sample) / self.channels as usize;

        let bytes = frames.as_bytes();
        let aligned = AlignedI16Data::new(bytes);

        let confidence = Python::with_gil(|py| {
            py.allow_threads(move || unsafe {
                daily_core_context_vad_analyze(
                    self.webrtc_vad.as_ref().unwrap().as_ptr() as *mut _,
                    aligned.as_ptr(),
                    num_frames,
                )
            })
        });

        Ok(confidence)
    }
}
