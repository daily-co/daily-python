use pyo3::prelude::*;

/// This class represents received audio data. It contains a bytestring with the
/// audio frames and other attributes such as bits per sample and sample rate.
#[pyclass(name = "AudioData", module = "daily", get_all, unsendable)]
pub struct PyAudioData {
    /// The bits per sample of the audio data
    pub bits_per_sample: i32,
    /// The sample rate
    pub sample_rate: i32,
    /// The number of audio channels
    pub num_channels: usize,
    /// The number of audio frames
    pub num_audio_frames: usize,
    /// A bytestring with the audio frames
    pub audio_frames: PyObject,
}
