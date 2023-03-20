// https://content-blockchain.org/research/testing-different-image-hash-functions/
// https://github.com/JohannesBuchner/imagehash
// https://apiumhub.com/tech-blog-barcelona/introduction-perceptual-hashes-measuring-similarity/

//use crate::engine::status_mgr::StatusMgrMsg::{AddActiveHasher, HasherFinished};
use crate::engine::first_or_default;
use crate::engine::status_mgr::{ImageData, StatusMgr};
use crossbeam_channel::{Receiver, Sender};
use image::DynamicImage;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::engine::hasher::ahasher::AHasher;
use crate::engine::hasher::dhasher::DHasher;

mod ahasher;
mod dhasher;

pub trait Hasher : Send + Sync + 'static {
    fn hash(&self, image: &DynamicImage) -> u64;
    fn hash_getter(&self, status_mgr: &impl AsRef<StatusMgr>, path: &impl AsRef<PathBuf>) -> Option<u64>;
    fn hash_setter(&self, status_mgr: &mut StatusMgr, path: &impl AsRef<PathBuf>, hsh: u64);
}

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
    status_mgr: Arc<Mutex<StatusMgr>>,
) -> Receiver<()> {
    let (done_sender, done_receiver) = crossbeam_channel::bounded(1);

    thread::spawn(move || {
        let (ahash_sndr, ahash_done_recv) = start_hasher(
            AHasher,
            status_mgr.clone(),
        );
        let (dhash_sndr, dhash_done_recv) = start_hasher(
            DHasher,
            status_mgr.clone(),
        );

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

        status_mgr.lock().unwrap().finish_scan();
        done_sender.send(()).unwrap_or_default();
    });

    done_receiver
}

fn start_hasher(
    hasher: impl Hasher,
    status_mgr: Arc<Mutex<StatusMgr>>,
) -> (Sender<(Arc<PathBuf>, Arc<DynamicImage>)>, Receiver<()>) {
    let (image_sender, image_receiver) : (Sender<(Arc<PathBuf>, Arc<DynamicImage>)>, Receiver<(Arc<PathBuf>, Arc<DynamicImage>)>)
        = crossbeam_channel::bounded(10);
    let (done_sender, done_receiver) = crossbeam_channel::bounded(1);

    thread::spawn(move || {
        for (pathbuf, shared_image) in image_receiver {
            let hash = hasher.hash(&shared_image.as_ref());
            hasher.hash_setter(&mut *status_mgr.lock().unwrap(), &pathbuf, hash);
        }
        done_sender.send(())
    });

    (image_sender, done_receiver)
}
