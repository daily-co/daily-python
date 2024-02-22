use webrtc_daily::sys::{
    color_format::ColorFormat, virtual_camera_device::NativeVirtualCameraDevice,
};

use daily_core::prelude::daily_core_context_virtual_camera_device_write_frame;

use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// This class represents a virtual camera device. Virtual camera
/// devices are used to send video to the meeting.
#[pyclass(name = "VirtualCameraDevice", module = "daily")]
pub struct PyVirtualCameraDevice {
    device_name: String,
    width: u32,
    height: u32,
    color_format: ColorFormat,
    camera_device: Option<NativeVirtualCameraDevice>,
}

impl PyVirtualCameraDevice {
    pub fn new(device_name: &str, width: u32, height: u32, color_format: ColorFormat) -> Self {
        Self {
            device_name: device_name.to_string(),
            width,
            height,
            color_format,
            camera_device: None,
        }
    }

    pub fn attach_camera_device(&mut self, camera_device: NativeVirtualCameraDevice) {
        self.camera_device = Some(camera_device);
    }
}

#[pymethods]
impl PyVirtualCameraDevice {
    /// Returns the device name.
    ///
    /// :return: The virtual camera device name
    /// :rtype: str
    #[getter]
    fn name(&self) -> String {
        self.device_name.clone()
    }

    /// Returns the resolution width of this camera.
    ///
    /// :return: The resolution width
    /// :rtype: int
    #[getter]
    fn width(&self) -> u32 {
        self.width
    }

    /// Returns the resolution height of this camera.
    ///
    /// :return: The resolution height
    /// :rtype: int
    #[getter]
    fn height(&self) -> u32 {
        self.height
    }

    /// Returns the color format of this camera.
    ///
    /// :return: See :ref:`ColorFormat`
    /// :rtype: str
    #[getter]
    fn color_format(&self) -> String {
        self.color_format.to_string()
    }

    /// Writes a video frame to a virtual camera device created with
    /// :func:`Daily.create_camera_device`.
    ///
    /// The video frame needs to be of the same color format (see
    /// :ref:`ColorFormat`) specified when creating the camera.
    ///
    /// :param bytestring frame: A bytestring with the video frame contents
    pub fn write_frame(&self, py: Python<'_>, frame: &PyBytes) -> PyResult<()> {
        if let Some(camera_device) = self.camera_device.as_ref() {
            let bytes_length = frame.len()?;

            let bytes = frame.as_bytes();

            py.allow_threads(move || unsafe {
                daily_core_context_virtual_camera_device_write_frame(
                    camera_device.as_ptr() as *mut _,
                    bytes.as_ptr() as *const _,
                    bytes_length,
                )
            });

            Ok(())
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "no camera device has been attached",
            ))
        }
    }
}
