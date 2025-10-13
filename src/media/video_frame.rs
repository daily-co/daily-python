use pyo3::prelude::*;

/// This class represents a received video frame. It contains a bytestring with
/// frame contents and other frame attributes such as width and height.
#[pyclass(name = "VideoFrame", module = "daily", get_all)]
pub struct PyVideoFrame {
    /// A bytestring with the frame data in the corresponding color format
    pub buffer: Py<PyAny>,
    /// The width of this frame
    pub width: i32,
    /// The height this frame
    pub height: i32,
    /// The time in microseconds that the frame was received
    pub timestamp_us: i64,
    /// The frame's color format
    pub color_format: Py<PyAny>,
}
