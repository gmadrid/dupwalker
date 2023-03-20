use std::path::PathBuf;
use std::sync::Arc;
use image::imageops::FilterType;
use image::DynamicImage;
use crate::engine::hasher::Hasher;
use crate::engine::status_mgr::StatusMgr;

pub struct DHasher;

impl Hasher for DHasher {
    fn hash(&self, image: &DynamicImage) -> u64 {
        dhash(image)
    }

    fn hash_getter(&self, status_mgr: &impl AsRef<StatusMgr>, path: &impl AsRef<PathBuf>) -> Option<u64> {
        let image_data = status_mgr.as_ref().get_image_data(path.as_ref());
        image_data.and_then(|id| id.d_hash)
    }

    fn hash_setter(&self, status_mgr: &mut StatusMgr, path: &impl AsRef<PathBuf>, hsh: u64) {
        // TODO: this is awkward. Stop passing Arc<PathBuf> unless you can do it all the way.
        status_mgr.dhash(Arc::new(path.as_ref().clone()), hsh);
    }
}

pub fn dhash(image: &DynamicImage) -> u64 {
    let reduced = image.resize_exact(9, 8, FilterType::Gaussian).to_luma16();
    let mut hash = 0u64;

    for row in 0..8 {
        for col in 0..8 {
            hash <<= 1;
            if reduced.get_pixel(col, row)[0] > reduced.get_pixel(col + 1, row)[0] {
                hash += 1;
            }
        }
    }

    hash
}
