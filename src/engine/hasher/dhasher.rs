use crate::engine::status_mgr::StatusMgrMsg;
use crate::engine::status_mgr::StatusMgrMsg::{DHash};
use crossbeam_channel::{Sender};
use image::imageops::FilterType;
use image::DynamicImage;
use std::path::PathBuf;
use std::thread;

pub fn start(status_sndr: Sender<StatusMgrMsg>) -> Sender<(PathBuf, DynamicImage)> {
    let (sender, receiver) = crossbeam_channel::bounded(10);

    thread::spawn(move || {
        for (pathbuf, image) in receiver {
            let hash = dhash(&image);

            status_sndr.send(DHash(pathbuf, hash)).unwrap();
        }
    });

    sender
}

fn dhash(image: &DynamicImage) -> u64 {
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
