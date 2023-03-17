use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize)]
pub struct DWStatus {
    pub count: usize,
    pub finished: bool,
    pub last_scanned: Option<PathBuf>,
}
