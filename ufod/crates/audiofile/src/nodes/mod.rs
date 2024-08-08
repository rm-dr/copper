//! Pipeline nodes for processing audio files

mod striptags;
pub use striptags::*;

mod extractcovers;
pub use extractcovers::*;

mod extracttags;
pub use extracttags::*;
