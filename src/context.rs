use thiserror::Error;

use webrtc_daily::media_stream::{MediaStream, MediaStreamError};

#[derive(Error, Debug)]
pub(crate) enum DailyError {
    #[error(transparent)]
    MediaStreamError(#[from] MediaStreamError),
}

pub(crate) struct DailyContext {
    pub media_stream: MediaStream,
}

impl DailyContext {
    pub fn new() -> Result<Self, DailyError> {
        let media_stream = MediaStream::new()?;
        Ok(Self { media_stream })
    }
}
