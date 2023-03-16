use crate::engine::status_mgr::StatusMgrMsg;
use crate::engine::status_mgr::StatusMgrMsg::NoOp;
use crossbeam_channel::{Receiver, Sender};
use image::DynamicImage;
use std::path::PathBuf;
use std::thread;

pub fn start(
    recv: Receiver<(PathBuf, DynamicImage)>,
    status_sndr: Sender<StatusMgrMsg>,
) -> Receiver<()> {
    let (sender, receiver) = crossbeam_channel::bounded(1);

    thread::spawn(move || {
        for (_, _) in recv {
            status_sndr.send(NoOp).unwrap();
        }
        sender.send(()).unwrap();
    });

    receiver
}
