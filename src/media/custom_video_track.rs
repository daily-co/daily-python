use daily_core::prelude::*;

use webrtc_daily::media_stream::MediaStreamTrack;

use pyo3::prelude::*;
use webrtc_daily::sys::webrtc::MediaStreamTrackInterface;
use webrtc_daily::sys::ScopedRefPtr;

use super::PyCustomVideoSource;

/// This class represents a custom video track. Custom video tracks need a
/// :class:`CustomVideoSource` to write video frames.
///
/// Custom video tracks can be used to send additional custom tracks or as the
/// main camera track.
#[pyclass(name = "CustomVideoTrack", module = "daily")]
pub struct PyCustomVideoTrack {
    pub video_track: MediaStreamTrack,
}

#[pymethods]
impl PyCustomVideoTrack {
    #[new]
    pub fn new(video_source: &PyCustomVideoSource) -> Self {
        let video_track = unsafe {
            daily_core_context_create_custom_video_track(
                video_source.video_source.as_ptr() as *mut _
            )
        };

        let video_track = video_track as *mut MediaStreamTrackInterface;

        let video_track =
            unsafe { MediaStreamTrack::from(ScopedRefPtr::from_retained(video_track)) };

        Self { video_track }
    }

    /// Returns the track id.
    ///
    /// :return: The track id
    /// :rtype: str
    #[getter]
    fn id(&self) -> String {
        self.video_track.id()
    }
}

impl Drop for PyCustomVideoTrack {
    fn drop(&mut self) {
        unsafe {
            daily_core_context_destroy_custom_video_track(self.video_track.as_mut_ptr() as *mut _)
        };
    }
}
