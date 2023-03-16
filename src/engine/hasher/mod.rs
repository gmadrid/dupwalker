// https://content-blockchain.org/research/testing-different-image-hash-functions/
// https://github.com/JohannesBuchner/imagehash
// https://apiumhub.com/tech-blog-barcelona/introduction-perceptual-hashes-measuring-similarity/

use crate::engine::status_mgr::StatusMgrMsg;
use crossbeam_channel::{Receiver, Sender};
use image::DynamicImage;
use std::path::PathBuf;
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
            dhash_sndr.send((path.clone(), image.clone())).unwrap();
            ahash_sndr.send((path.clone(), image.clone())).unwrap();
        }

        sender.send(()).unwrap();
    });

    receiver
}
