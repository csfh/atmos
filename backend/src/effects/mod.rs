//! Platform documents and the tools that already own them.
//!
//! List domains go through the sentinel writers (`ratmos sentinel`). Other
//! domains write the file Omarchy's own tool writes, and a live (no `--root`)
//! set also runs that tool.
//!
//! One module per document family: `hypr` (sentinel blocks), `docs` (the
//! dispatch over every other document), then `sunset`, `env`, `mime`, `audio`
//! and `system` for each format, and `live` for the commands that run after a
//! write.

use crate::error::{Error, Result};
use crate::fsutil::atomic_write_text as atomic_text;
use crate::runner::Run;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Map, Value};

mod audio;
mod docs;
mod env;
mod hypr;
mod live;
mod mime;
mod sunset;
mod system;

pub use self::docs::{read_doc, write_doc};
pub use self::hypr::{read_hypr, write_hypr};
pub use self::live::after_write;
pub(crate) use self::live::spawn;

use self::audio::*;
use self::env::*;
use self::mime::*;
use self::sunset::*;
use self::system::*;
