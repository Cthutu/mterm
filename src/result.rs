use thiserror::Error;
use winit::error::OsError;

use crate::RenderError;

/// All the possible errors that can occur from mterm.
///
/// Uses `thiserror` to generate new error enums, as well as pass over `winit`
/// and `wgpu` errors.
#[derive(Error, Debug)]
pub enum Error {
    /// An error occurred within `winit`.
    #[error(transparent)]
    WinitError(#[from] OsError),

    /// An error occurred within `wgpu`.
    #[error(transparent)]
    WgpuError(#[from] RenderError),

    #[error("Unable to read font data")]
    BadFont,
}

/// A result that can possible return an `mterm::Error`.
pub type Result<T> = std::result::Result<T, Error>;
