// https://content-blockchain.org/research/testing-different-image-hash-functions/
// https://github.com/JohannesBuchner/imagehash
// https://apiumhub.com/tech-blog-barcelona/introduction-perceptual-hashes-measuring-similarity/

use crate::engine::status_mgr::{ImageData, StatusMgrMsg};
use crossbeam_channel::{Receiver, Sender};
use image::DynamicImage;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use crate::engine::first_or_default;
use crate::engine::status_mgr::StatusMgrMsg::{AddActiveHasher, HasherFinished};

mod ahasher;
mod dhasher;

pub fn start(
    image_recv: Receiver<(Arc<PathBuf>, Arc<DynamicImage>, ImageData)>,
    status_sndr: Sender<StatusMgrMsg>,
) -> Receiver<()> {
    let (sender, receiver) = crossbeam_channel::bounded(1);

    thread::spawn(move || {
        let ahash_sndr = start_hasher(ahasher::ahash, StatusMgrMsg::AHash, status_sndr.clone());
        let dhash_sndr = start_hasher(dhasher::dhash, StatusMgrMsg::DHash, status_sndr.clone());

        for (shared_path, shared_image, image_data) in image_recv {
            if image_data.d_hash.is_none() {
                dhash_sndr
                    .send((shared_path.clone(), shared_image.clone()))
                    .unwrap();
            }
            if image_data.a_hash.is_none() {
                ahash_sndr
                    .send((shared_path.clone(), shared_image.clone()))
                    .unwrap();
            }
        }

        // We unwrap_or_default() in order to ignore any errors.
        sender.send(()).unwrap_or_default();
    });

    receiver
}

type HashFunc = fn(&DynamicImage) -> u64;
type MsgFunc = fn(Arc<PathBuf>, u64) -> StatusMgrMsg;

fn start_hasher(
    hash_func: HashFunc,
    msg_func: MsgFunc,
    status_sndr: Sender<StatusMgrMsg>,
) -> Sender<(Arc<PathBuf>, Arc<DynamicImage>)> {
    let (sender, receiver) = crossbeam_channel::bounded(10);

    thread::spawn(move || {
        status_sndr.send(AddActiveHasher).unwrap();
        for (pathbuf, shared_image) in receiver {
            let hash = hash_func(Arc::as_ref(&shared_image));
            status_sndr.send(msg_func(pathbuf, hash)).unwrap();
        }
        status_sndr.send(HasherFinished)
    });

    sender
}
