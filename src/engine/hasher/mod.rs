// https://content-blockchain.org/research/testing-different-image-hash-functions/
// https://github.com/JohannesBuchner/imagehash
// https://apiumhub.com/tech-blog-barcelona/introduction-perceptual-hashes-measuring-similarity/

use crate::engine::status_mgr::StatusMgrMsg;
use crossbeam_channel::{Receiver, Sender};
use image::DynamicImage;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

mod ahasher;
mod dhasher;
//mod fake_hasher;

pub fn start(
    image_recv: Receiver<(PathBuf, DynamicImage)>,
    status_sndr: Sender<StatusMgrMsg>,
) -> Receiver<()> {
    let (sender, receiver) = crossbeam_channel::bounded(1);

    thread::spawn(move || {
        let dhash_sndr = dhasher::start(status_sndr.clone());
        let ahash_sndr = ahasher::start(status_sndr.clone());

        for (path, image) in image_recv {
            let shared_path = Arc::new(path);
            let shared_image = Arc::new(image);
            dhash_sndr
                .send((shared_path.clone(), shared_image.clone()))
                .unwrap();
            ahash_sndr
                .send((shared_path.clone(), shared_image.clone()))
                .unwrap();
        }

        sender.send(()).unwrap();
    });

    receiver
}
