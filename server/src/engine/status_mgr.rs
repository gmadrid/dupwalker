use crossbeam_channel::Sender;
use shared::DWStatus;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::serde_json;

pub enum StatusMgrMsg {
    NoOp,
    AHash(Arc<PathBuf>, u64),
    DHash(Arc<PathBuf>, u64),
    GetImageData(Arc<PathBuf>, Sender<Option<ImageData>>),
    AddActiveHasher,
    HasherFinished,

    SaveData,

    StatusRequest(Sender<DWStatus>),
    TestMsg(Sender<String>),
}

#[derive(Default, Serialize, Deserialize)]
pub struct StatusMgr {
    data: HashMap<PathBuf, ImageData>,

    #[serde(skip)]
    active_hashers: usize,

    #[serde(skip)]
    last_scanned: Option<Arc<PathBuf>>,

    #[serde(skip)]
    dirty: bool,
}

impl StatusMgr {
    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        if let Ok(f) = std::fs::File::open(path) {
            // TODO: we are failing silently here. Maybe we shouldn't.
            serde_json::from_reader(f).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&mut self, path: impl AsRef<Path>) {
        let w = std::fs::File::create(path.as_ref()).unwrap();
        serde_json::to_writer_pretty(w, self).unwrap();
        self.dirty = false;

        println!("Saving {} entries at {:?}", self.data.len(), SystemTime::now());
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
        println!("Loaded cache with {} entries.", mgr.data.len());
        for msg in receiver {
            match msg {
                StatusMgrMsg::NoOp => {
                    println!("NoOp");
                }
                StatusMgrMsg::AHash(pathbuf, hsh) => {
                    // println!("AHash: {}: {:b}", pathbuf.file_name().unwrap_or_default().to_string_lossy(), hsh);
                    mgr.last_scanned = Some(pathbuf.clone());
                    mgr.data.entry(pathbuf.to_path_buf()).or_default().a_hash = Some(hsh);
                    mgr.dirty = true;
                    println!("AHASH: {}", mgr.data.len());
                }
                StatusMgrMsg::DHash(pathbuf, hsh) => {
                    // println!("DHash: {}: {:b}", pathbuf.file_name().unwrap_or_default().to_string_lossy(), hsh);
                    mgr.last_scanned = Some(pathbuf.clone());
                    mgr.data.entry(pathbuf.to_path_buf()).or_default().d_hash = Some(hsh);
                    mgr.dirty = true;
                    println!("DHASH: {}", mgr.data.len());
                }
                StatusMgrMsg::GetImageData(pathbuf, sndr) => {
                    sndr.send(mgr.data.get(pathbuf.as_ref()).cloned()).unwrap();
                }
                StatusMgrMsg::TestMsg(sndr) => {
                    let s = format!("{}", mgr.data.len());
                    sndr.send(s).unwrap();
                }
                StatusMgrMsg::StatusRequest(sndr) => {
                    let s = DWStatus {
                        count: mgr.data.len(),
                        finished: mgr.active_hashers == 0,
                        last_scanned: mgr.last_scanned.as_ref().map(|apb| apb.as_ref().clone()),
                    };
                    sndr.send(s).unwrap();
                }
                StatusMgrMsg::SaveData => {
                    mgr.save(&cache_file);
                }
                StatusMgrMsg::AddActiveHasher => {
                    mgr.active_hashers += 1;
                }
                StatusMgrMsg::HasherFinished => {
                    mgr.active_hashers -= 1;
                    if mgr.active_hashers == 0 {
                        mgr.save(&cache_file);
                        println!("FINISHED: {}", mgr.data.len());
                    }
                }
            }
        }
    });

    sender
}
