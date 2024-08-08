//! Flac processors. These are well-tested wrappers around [`crate::flac::blockread::FlacBlockReader`]
//! that are specialized for specific tasks.

pub mod metastrip;
pub mod pictures;
