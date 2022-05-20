//! Asynchronous [STAC](https://stacspec.org/).

#![deny(missing_docs, missing_debug_implementations)]

mod read;

pub use read::{AsyncRead, AsyncReader};
