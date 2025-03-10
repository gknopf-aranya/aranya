use core::{ffi::c_char, mem::MaybeUninit};

use aranya_buggy::Bug;
use aranya_capi_core::{
    safe::{TypeId, Typed},
    write_c_str, ExtendedError, InvalidArg, WriteCStrError,
};
use tracing::warn;
use tracing_subscriber::util::TryInitError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Bug(#[from] Bug),

    #[error(transparent)]
    Timeout(#[from] tokio::time::error::Elapsed),

    #[error("could not initialize logging: {0}")]
    LogInit(#[from] TryInitError),

    /// An invalid argument was provided.
    #[error(transparent)]
    InvalidArg(#[from] InvalidArg<'static>),

    #[error("buffer too small")]
    BufferTooSmall,

    #[error(transparent)]
    Utf8(#[from] core::str::Utf8Error),

    #[error("addr error: {0}")]
    Addr(#[from] aranya_util::AddrError),

    #[error("client error: {0}")]
    Client(#[from] aranya_client::Error),

    #[error("tokio runtime error: {0}")]
    Runtime(#[source] std::io::Error),
}

impl From<WriteCStrError> for Error {
    fn from(err: WriteCStrError) -> Self {
        match err {
            WriteCStrError::Bug(bug) => Self::Bug(bug),
            WriteCStrError::BufferTooSmall => Self::BufferTooSmall,
        }
    }
}

/// Underlying type for [`ExtError`][crate::api::ExtError].
#[derive(Default)]
pub struct ExtError {
    err: Option<Error>,
}

impl ExtError {
    /// Creates an `ExtError`.
    pub const fn new(err: Error) -> Self {
        Self { err: Some(err) }
    }

    /// Copies the error message to `msg` as a null-terminated
    /// C string.
    pub fn copy_msg(&self, msg: &mut [MaybeUninit<c_char>], len: &mut usize) -> Result<(), Error> {
        if let Some(err) = &self.err {
            write_c_str(msg, err, len).map_err(Into::into)
        } else {
            warn!("empty extended error empty");
            write_c_str(msg, &"", len).map_err(Into::into)
        }
    }
}

impl Typed for ExtError {
    const TYPE_ID: TypeId = TypeId::new(0xa2a040);
}

impl ExtendedError for ExtError {
    type Error = Error;

    fn set<E>(&mut self, err: Option<E>)
    where
        E: Into<Self::Error>,
    {
        self.err = err.map(Into::into)
    }
}
