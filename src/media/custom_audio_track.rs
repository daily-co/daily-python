use daily_core::prelude::*;

use webrtc_daily::media_stream::MediaStreamTrack;

use pyo3::prelude::*;
use webrtc_daily::sys::webrtc::MediaStreamTrackInterface;
use webrtc_daily::sys::ScopedRefPtr;

use super::PyCustomAudioSource;

/// This class represents a custom audio track. Custom audio tracks need a
/// :class:`CustomAudioSource` to write audio frames.
///
/// Custom audio tracks can be used to send additional custom tracks or as the
/// main microphone track.
#[pyclass(name = "CustomAudioTrack", module = "daily")]
pub struct PyCustomAudioTrack {
    pub audio_track: MediaStreamTrack,
}

#[pymethods]
impl PyCustomAudioTrack {
    #[new]
    pub fn new(audio_source: &PyCustomAudioSource) -> Self {
        let audio_track = unsafe {
            daily_core_context_create_custom_audio_track(
                audio_source.audio_source.as_ptr() as *mut _
            )
        };

        let audio_track = audio_track as *mut MediaStreamTrackInterface;

        let audio_track =
            unsafe { MediaStreamTrack::from(ScopedRefPtr::from_retained(audio_track)) };

        Self { audio_track }
    }

    /// Returns the track id.
    ///
    /// :return: The track id
    /// :rtype: str
    #[getter]
    fn id(&self) -> String {
        self.audio_track.id()
    }
}

impl Drop for PyCustomAudioTrack {
    fn drop(&mut self) {
        unsafe {
            daily_core_context_destroy_custom_audio_track(self.audio_track.as_mut_ptr() as *mut _)
        };
    }
}
