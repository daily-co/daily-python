use std::sync::atomic::{AtomicU64, Ordering};

use thiserror::Error;

use webrtc_daily::media_stream::{MediaStream, MediaStreamError};

// This should be initialized from Daily.init().
// TODO(aleix): Is this even the way to do this? Do we need a global context?
pub static mut GLOBAL_CONTEXT: Option<DailyContext> = None;

#[derive(Error, Debug)]
pub enum DailyError {
    #[error(transparent)]
    MediaStreamError(#[from] MediaStreamError),
}

pub struct DailyContext {
    request_id: AtomicU64,
    media_stream: MediaStream,
}

impl DailyContext {
    pub fn new() -> Result<Self, DailyError> {
        let media_stream = MediaStream::new()?;
        Ok(Self {
            request_id: AtomicU64::new(0),
            media_stream,
        })
    }

    pub fn media_stream(&self) -> MediaStream {
        self.media_stream.clone()
    }

    pub fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }
}
