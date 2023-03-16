use crossbeam_channel::Sender;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

pub enum StatusMgrMsg {
    //NoOp,
    AHash(Arc<PathBuf>, u64),
    DHash(Arc<PathBuf>, u64),
}

#[derive(Default)]
pub struct StatusMgr {
    data: HashMap<Arc<PathBuf>, ImageData>,
}

#[derive(Default, Debug)]
pub struct ImageData {
    a_hash: Option<u64>,
    d_hash: Option<u64>,
}

pub fn start() -> Sender<StatusMgrMsg> {
    let (sender, receiver) = crossbeam_channel::bounded(10);

    thread::spawn(move || {
        let mut mgr = StatusMgr::default();
        for msg in receiver {
            match msg {
                StatusMgrMsg::AHash(pathbuf, hsh) => {
                    // println!("AHash: {}: {:b}", pathbuf.file_name().unwrap_or_default().to_string_lossy(), hsh);
                    mgr.data.entry(pathbuf).or_default().a_hash = Some(hsh);
                }
                StatusMgrMsg::DHash(pathbuf, hsh) => {
                    // println!("DHash: {}: {:b}", pathbuf.file_name().unwrap_or_default().to_string_lossy(), hsh);
                    mgr.data.entry(pathbuf).or_default().d_hash = Some(hsh);
                }
            }
        }
    });

    sender
}
