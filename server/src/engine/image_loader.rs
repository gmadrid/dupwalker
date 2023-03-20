use crate::engine::status_mgr::{ImageData, StatusMgr};
use crossbeam_channel::Receiver;
use image::DynamicImage;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

pub(crate) fn start(
    file_recv: Receiver<PathBuf>,
    status_mgr: Arc<Mutex<StatusMgr>>,
) -> Receiver<(Arc<PathBuf>, Arc<DynamicImage>, ImageData)> {
    let (sender, receiver) = crossbeam_channel::bounded(100);

    thread::spawn(move || {
        for filename in file_recv {
            let shared_path = Arc::new(filename);
            let image_data = status_mgr
                .lock()
                .unwrap()
                .get_image_data(shared_path.as_ref())
                .cloned()
                .unwrap_or_default();
            if !image_data.is_complete() {
                if let Ok(img) = image::open(shared_path.as_ref()) {
                    let gray = img.grayscale();
                    sender
                        .send((shared_path, Arc::new(gray), image_data))
                        .unwrap();
                }
            }
        }
    });

    receiver
}
