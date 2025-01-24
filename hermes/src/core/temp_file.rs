use std::path::Path;

use tempfile::NamedTempFile;

pub struct TempFile {
    pub _file: NamedTempFile,
    pub path: &'static Path,
}
