//! Platform documents and the scripts that already own them.
//!
//! List domains go through `hypr-sentinel.py`, so the sentinel body is the
//! same Lua that script writes. Other domains write the file the existing
//! shell script writes, and a live (no `--root`) set also runs that script.
//!
//! One module per document family: `hypr` (sentinel blocks), `docs` (the
//! dispatch over every other document), then `sunset`, `env`, `mime`, `audio`
//! and `system` for each format, and `live` for the commands that run after a
//! write.

use crate::error::{Error, Result};
use crate::fsutil::atomic_write_text as atomic_text;
use crate::runner::{Run, COMMAND_TIMEOUT};
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

use self::audio::*;
use self::env::*;
use self::live::*;
use self::mime::*;
use self::sunset::*;
use self::system::*;
