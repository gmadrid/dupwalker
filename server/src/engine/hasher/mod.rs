// https://content-blockchain.org/research/testing-different-image-hash-functions/
// https://github.com/JohannesBuchner/imagehash
// https://apiumhub.com/tech-blog-barcelona/introduction-perceptual-hashes-measuring-similarity/

//use crate::engine::status_mgr::StatusMgrMsg::{AddActiveHasher, HasherFinished};
use crate::engine::first_or_default;
use crate::engine::status_mgr::{ImageData, StatusMgrMsg};
use crossbeam_channel::{Receiver, Sender};
use image::DynamicImage;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

mod ahasher;
mod dhasher;

/// Start the Hasher on another thread.
///
/// Args:
///   image_recv: A Receiver sending the path to an image, the Image at that path, and the ImageData
///               as of the start of this call.
///   status_sndr: Sender for interaction with the StatusMgr.
/// Returns a Receiver that will receive a single item when all Hashers have finished.
///
pub fn start(
    image_recv: Receiver<(Arc<PathBuf>, Arc<DynamicImage>, ImageData)>,
    status_sndr: Sender<StatusMgrMsg>,
) -> Receiver<()> {
    let (done_sender, done_receiver) = crossbeam_channel::bounded(1);

    thread::spawn(move || {
        let (ahash_sndr, ahash_done_recv) =
            start_hasher(ahasher::ahash, StatusMgrMsg::AHash, status_sndr.clone());
        let (dhash_sndr, dhash_done_recv) =
            start_hasher(dhasher::dhash, StatusMgrMsg::DHash, status_sndr.clone());

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

        // drop the senders so that the hashers will stop waiting,
        // then wait for the hashers to be finished.
        drop(ahash_sndr);
        first_or_default(ahash_done_recv);
        drop(dhash_sndr);
        first_or_default(dhash_done_recv);

        // We unwrap_or_default() in order to ignore any errors.
        status_sndr
            .send(StatusMgrMsg::ScanFinished)
            .unwrap_or_default();
        done_sender.send(()).unwrap_or_default();
    });

    done_receiver
}

type HashFunc = fn(&DynamicImage) -> u64;
type MsgFunc = fn(Arc<PathBuf>, u64) -> StatusMgrMsg;

fn start_hasher(
    hash_func: HashFunc,
    msg_func: MsgFunc,
    status_sndr: Sender<StatusMgrMsg>,
) -> (Sender<(Arc<PathBuf>, Arc<DynamicImage>)>, Receiver<()>) {
    let (image_sender, image_receiver) = crossbeam_channel::bounded(10);
    let (done_sender, done_receiver) = crossbeam_channel::bounded(1);

    thread::spawn(move || {
        for (pathbuf, shared_image) in image_receiver {
            let hash = hash_func(Arc::as_ref(&shared_image));
            status_sndr.send(msg_func(pathbuf, hash)).unwrap();
        }
        done_sender.send(())
    });

    (image_sender, done_receiver)
}
