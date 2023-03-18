use crossbeam_channel::{Receiver, Sender};
use image::DynamicImage;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use crate::engine::first_or_default;
use crate::engine::status_mgr::{ImageData, StatusMgrMsg};

pub(crate) fn start(file_recv: Receiver<PathBuf>, status_sndr: Sender<StatusMgrMsg>) -> Receiver<(Arc<PathBuf>, Arc<DynamicImage>, ImageData)> {
    let (sender, receiver) = crossbeam_channel::bounded(100);

    thread::spawn(move || {
        for filename in file_recv {
            let shared_path = Arc::new(filename);
            let (qsndr, qrecv) = crossbeam_channel::bounded(1);
            status_sndr.send(StatusMgrMsg::GetImageData(shared_path.clone(), qsndr)).unwrap();

            let image_data = first_or_default(qrecv).unwrap_or_default();
            if !image_data.is_complete() {
                if let Ok(img) = image::open(shared_path.as_ref()) {
                    let gray = img.grayscale();
                    sender.send((shared_path, Arc::new(gray), image_data)).unwrap();
                }
            }
        }
    });

    receiver
}
