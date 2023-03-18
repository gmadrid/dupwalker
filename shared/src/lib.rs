use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize)]
pub enum DWStatus {
    #[default]
    Initializing,
    Scanning(DWScanningStatus),
    Comparing(DWComparingStatus),
    Ready,
}

#[derive(Default, Serialize, Deserialize)]
pub struct DWScanningStatus {
    pub count: usize,
    pub last_scanned: Option<PathBuf>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct DWComparingStatus {
    pub total_images: usize,
    pub image_scanning: usize,
}
