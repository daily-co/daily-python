use std::ffi::CString;
use std::str::FromStr;

use daily_core::prelude::*;

use webrtc_daily::sys::color_format::ColorFormat;
use webrtc_daily::sys::custom_video_source::NativeDailyVideoSource;

use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// This class represents a custom video source. Custom video sources are used
/// to send video frames to a video track. See
/// :func:`daily.CallClient.add_custom_video_track`.
#[pyclass(name = "CustomVideoSource", module = "daily")]
pub struct PyCustomVideoSource {
    pub video_source: NativeDailyVideoSource,
    width: u32,
    height: u32,
    color_format: ColorFormat,
}

impl PyCustomVideoSource {
    fn bytes_per_pixel(&self) -> usize {
        match self.color_format {
            ColorFormat::RGBA | ColorFormat::BGRA | ColorFormat::ARGB | ColorFormat::ABGR => 4,
            ColorFormat::RGB => 3,
            ColorFormat::I420 => 1, // not a simple bpp, but we handle the size check differently
        }
    }

    fn expected_frame_size(&self) -> usize {
        let w = self.width as usize;
        let h = self.height as usize;
        match self.color_format {
            ColorFormat::I420 => w * h * 3 / 2,
            _ => w * h * self.bytes_per_pixel(),
        }
    }
}

#[pymethods]
impl PyCustomVideoSource {
    /// Creates a custom video source.
    ///
    /// :param int width: Resolution width
    /// :param int height: Resolution height
    /// :param str color_format: The color format of the frames that will be written. See :ref:`ColorFormat`
    #[new]
    #[pyo3(signature = (width, height, color_format = "RGBA"))]
    pub fn new(width: u32, height: u32, color_format: &str) -> PyResult<Self> {
        let color_format = ColorFormat::from_str(color_format).map_err(|_| {
            exceptions::PyValueError::new_err(format!(
                "invalid color format '{}'. Valid formats: RGBA, BGRA, ARGB, ABGR, RGB, I420",
                color_format
            ))
        })?;

        let video_source_ptr = unsafe { daily_core_context_create_custom_video_source() };

        let video_source = NativeDailyVideoSource::from(video_source_ptr);

        Ok(Self {
            video_source,
            width,
            height,
            color_format,
        })
    }

    /// Returns the resolution width.
    ///
    /// :return: The resolution width
    /// :rtype: int
    #[getter]
    fn width(&self) -> u32 {
        self.width
    }

    /// Returns the resolution height.
    ///
    /// :return: The resolution height
    /// :rtype: int
    #[getter]
    fn height(&self) -> u32 {
        self.height
    }

    /// Returns the color format.
    ///
    /// :return: See :ref:`ColorFormat`
    /// :rtype: str
    #[getter]
    fn color_format(&self) -> String {
        self.color_format.to_string()
    }

    /// Writes a video frame to the video source. The frame will be sent to the
    /// video track if this source is attached to a track.
    ///
    /// The video frame needs to be of the same color format specified when
    /// creating the source.
    ///
    /// :param bytestring frame: A bytestring with the video frame contents
    pub fn write_frame(&self, py: Python<'_>, frame: &Bound<'_, PyBytes>) -> PyResult<()> {
        let bytes_length = frame.len()?;
        let expected_size = self.expected_frame_size();

        if bytes_length != expected_size {
            return Err(exceptions::PyValueError::new_err(format!(
                "frame size mismatch: expected {} bytes for {}x{} {}, got {}",
                expected_size, self.width, self.height, self.color_format, bytes_length
            )));
        }

        let bytes = frame.as_bytes();

        let color_format_str =
            CString::new(self.color_format.to_string()).expect("invalid color format string");

        tracing::trace!(
            "Writing video frame to video source {:?} ({bytes_length} bytes)",
            self.video_source.as_ptr()
        );

        py.detach(move || unsafe {
            daily_core_context_custom_video_source_write_frame(
                self.video_source.as_ptr() as *mut _,
                bytes.as_ptr() as *const _,
                bytes_length,
                self.width as i32,
                self.height as i32,
                color_format_str.as_ptr(),
            )
        });

        Ok(())
    }
}
