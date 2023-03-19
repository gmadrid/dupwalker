use crossbeam_channel::Sender;
use rocket::serde::json::serde_json;
use rocket::serde::{Deserialize, Serialize};
use shared::{DWComparingStatus, DWScanningStatus, DWStatus};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;

pub enum StatusMgrMsg {
    NoOp,
    AHash(Arc<PathBuf>, u64),
    DHash(Arc<PathBuf>, u64),
    GetImageData(Arc<PathBuf>, Sender<Option<ImageData>>),
    ScanFinished,

    SaveData,

    StatusRequest(Sender<DWStatus>),
    FilesRequest(Sender<Vec<PathBuf>>),
}

#[derive(Default, Serialize, Deserialize)]
pub struct StatusMgr {
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
    fn ahash(&mut self, pathbuf: Arc<PathBuf>, hsh: u64) {
        self.last_scanned = Some(pathbuf.clone());
        self.data.entry(pathbuf.to_path_buf()).or_default().a_hash = Some(hsh);
        self.dirty = true;
    }

    fn dhash(&mut self, pathbuf: Arc<PathBuf>, hsh: u64) {
        self.last_scanned = Some(pathbuf.clone());
        self.data.entry(pathbuf.to_path_buf()).or_default().d_hash = Some(hsh);
        self.dirty = true;
    }

    fn finish_scan(&mut self) {
        self.scan_finished = true;
        self.save();
    }

    fn get_image_data(&self, path: &PathBuf) -> Option<&ImageData> {
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

pub fn start(cache_file: &Path) -> Sender<StatusMgrMsg> {
    let (sender, receiver) = crossbeam_channel::bounded(10);

    let cache_file = cache_file.to_path_buf();
    thread::spawn(move || {
        let mut mgr = StatusMgr::load_or_default(&cache_file);
        println!(
            "Loaded cache with {} entries from {}.",
            mgr.data.len(),
            cache_file.display()
        );
        for msg in receiver {
            match msg {
                StatusMgrMsg::NoOp => println!("NoOp"),
                StatusMgrMsg::AHash(pathbuf, hsh) => mgr.ahash(pathbuf.clone(), hsh),
                StatusMgrMsg::DHash(pathbuf, hsh) => mgr.dhash(pathbuf.clone(), hsh),
                StatusMgrMsg::SaveData => mgr.save(),
                StatusMgrMsg::ScanFinished => mgr.finish_scan(),

                StatusMgrMsg::FilesRequest(sndr) => {
                    let v = mgr.data.keys().cloned().collect();
                    sndr.send(v).unwrap();
                }
                StatusMgrMsg::GetImageData(pathbuf, sndr) => {
                    let image_data = mgr.get_image_data(pathbuf.as_ref()).cloned();
                    sndr.send(image_data).unwrap();
                }
                StatusMgrMsg::StatusRequest(sndr) => {
                    let s = if mgr.scan_finished {
                        DWStatus::Comparing(DWComparingStatus {
                            total_images: mgr.data.len(),
                            image_scanning: 0,
                        })
                    } else {
                        DWStatus::Scanning(DWScanningStatus {
                            count: mgr.data.len(),
                            last_scanned: mgr.last_scanned.as_ref().map(|apb| apb.as_ref().clone()),
                        })
                    };
                    sndr.send(s).unwrap();
                }
            }
        }
    });

    sender
}
