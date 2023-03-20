use rocket::serde::json::serde_json;
use rocket::serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Default, Serialize, Deserialize)]
pub struct StatusMgr {
    // TODO: replace this with hashbrown so that you can use Arc<PathBuf>?
    data: HashMap<PathBuf, ImageData>,

    #[serde(skip)]
    scan_finished: bool,

    #[serde(skip)]
    last_scanned: Option<Arc<PathBuf>>,

    #[serde(skip)]
    dirty: bool,

    #[serde(skip)]
    cache_path: PathBuf,
}

impl StatusMgr {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn paths(&self) -> Vec<PathBuf> {
        self.data.keys().cloned().collect()
    }

    pub fn ahash(&mut self, pathbuf: Arc<PathBuf>, hsh: u64) {
        self.last_scanned = Some(pathbuf.clone());
        self.data.entry(pathbuf.to_path_buf()).or_default().a_hash = Some(hsh);
        self.dirty = true;
    }

    pub fn dhash(&mut self, pathbuf: Arc<PathBuf>, hsh: u64) {
        self.last_scanned = Some(pathbuf.clone());
        self.data.entry(pathbuf.to_path_buf()).or_default().d_hash = Some(hsh);
        self.dirty = true;
    }

    pub fn last_scanned(&self) -> Option<Arc<PathBuf>> {
        self.last_scanned.clone()
    }

    pub fn scan_finished(&self) -> bool {
        self.scan_finished
    }

    pub fn finish_scan(&mut self) {
        self.scan_finished = true;
        self.save();
    }

    pub fn get_image_data(&self, path: &PathBuf) -> Option<&ImageData> {
        self.data.get(path)
    }

    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        let mut value = if let Ok(f) = std::fs::File::open(&path) {
            // TODO: we are failing silently here. Maybe we shouldn't.
            serde_json::from_reader(f).unwrap_or_default()
        } else {
            Self::default()
        };
        value.cache_path = path.as_ref().to_path_buf();
        value
    }

    pub fn save(&mut self) {
        let w = std::fs::File::create(&self.cache_path).unwrap();
        serde_json::to_writer_pretty(w, self).unwrap();
        self.dirty = false;

        println!(
            "Saving {} entries at {:?} to {:?}",
            self.data.len(),
            SystemTime::now(),
            &self.cache_path
        );
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct ImageData {
    pub a_hash: Option<u64>,
    pub d_hash: Option<u64>,
}

impl ImageData {
    pub fn is_complete(&self) -> bool {
        self.a_hash.is_some() && self.d_hash.is_some()
    }
}
