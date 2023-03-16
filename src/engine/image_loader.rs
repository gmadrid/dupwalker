use crossbeam_channel::Receiver;
use image::DynamicImage;
use std::path::PathBuf;
use std::thread;

pub(crate) fn start(file_recv: Receiver<PathBuf>) -> Receiver<(PathBuf, DynamicImage)> {
    let (sender, receiver) = crossbeam_channel::bounded(100);

    thread::spawn(move || {
        for filename in file_recv {
            if let Ok(img) = image::open(&filename) {
                let gray = img.grayscale();
                sender.send((filename, gray)).unwrap();
            }
        }
    });

    receiver
}
