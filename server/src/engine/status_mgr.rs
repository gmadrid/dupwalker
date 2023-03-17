use crossbeam_channel::Sender;
use shared::DWStatus;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

pub enum StatusMgrMsg {
    NoOp,
    AHash(Arc<PathBuf>, u64),
    DHash(Arc<PathBuf>, u64),

    ScanFinished,
    StatusRequest(Sender<DWStatus>),
    TestMsg(Sender<String>),
}

#[derive(Default)]
pub struct StatusMgr {
    data: HashMap<Arc<PathBuf>, ImageData>,
    finished: bool,
    last_scanned: Option<Arc<PathBuf>>,
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
                StatusMgrMsg::NoOp => {
                    println!("NoOp");
                }
                StatusMgrMsg::AHash(pathbuf, hsh) => {
                    // println!("AHash: {}: {:b}", pathbuf.file_name().unwrap_or_default().to_string_lossy(), hsh);
                    mgr.last_scanned = Some(pathbuf.clone());
                    mgr.data.entry(pathbuf).or_default().a_hash = Some(hsh);
                }
                StatusMgrMsg::DHash(pathbuf, hsh) => {
                    // println!("DHash: {}: {:b}", pathbuf.file_name().unwrap_or_default().to_string_lossy(), hsh);
                    mgr.last_scanned = Some(pathbuf.clone());
                    mgr.data.entry(pathbuf).or_default().d_hash = Some(hsh);
                }
                StatusMgrMsg::ScanFinished => {
                    mgr.finished = true;
                }
                StatusMgrMsg::TestMsg(sndr) => {
                    let s = format!("{}", mgr.data.len());
                    sndr.send(s).unwrap();
                }
                StatusMgrMsg::StatusRequest(sndr) => {
                    let s = DWStatus {
                        count: mgr.data.len(),
                        finished: mgr.finished,
                        last_scanned: mgr.last_scanned.as_ref().map(|apb| apb.as_ref().clone()),
                    };
                    sndr.send(s).unwrap();
                }
            }
        }
    });

    sender
}
