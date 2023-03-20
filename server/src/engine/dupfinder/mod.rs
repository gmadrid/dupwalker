use crate::engine::first_or_default;
use crate::engine::status_mgr::StatusMgrMsg::GetImageData;
use crate::engine::status_mgr::{ImageData, StatusMgrMsg};
use crossbeam_channel::{Receiver, Sender};
use std::cmp::{max, min};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

struct DupFinderMap<G: Fn(&ImageData) -> Option<u64>> {
    hash_getter: G,
    // TODO: would this work better as a HashMap?
    lookup: [Vec<FileInfo>; 65],
}

fn count_bits(mut n: u64) -> u8 {
    let mut count = 0u8;
    for _ in std::iter::repeat(()).take(64) {
        if n % 2 == 1 {
            count += 1;
        }
        n >>= 1;
    }
    count
}

impl<G: Fn(&ImageData) -> Option<u64>> DupFinderMap<G> {
    fn new(hash_getter: G) -> DupFinderMap<G> {
        DupFinderMap {
            hash_getter,
            lookup: [(); 65].map(|_| vec![]),
        }
    }

    pub fn add_image(&mut self, path: &Path, image_data: &ImageData) -> Option<u64>{
        if let Some(hash) = (self.hash_getter)(image_data) {
            let bit_count = count_bits(hash);
            let v = &mut self.lookup[bit_count as usize];
            v.push(FileInfo {
                filename: path.to_path_buf(),
                hash,
            });
            Some(hash)
        } else {
            None
        }
    }

    pub fn find_near_images(&self, hash: u64, distance: u8) -> Vec<PathBuf> {
        let center = count_bits(hash);
        let bottom = max(0, center as i8 - distance as i8) as usize;
        let top = min(64, center + distance) as usize;

        let mut found = Vec::<PathBuf>::default();
        for idx in bottom..=top {
            let list = &self.lookup[idx];
            for fi in list {
                let xored = !(hash ^ fi.hash);
                let hamming = count_bits(xored);
                if hamming <= distance {
                    found.push(fi.filename.clone());
                }
            }
        }
        found
    }
}

#[derive(Default)]
struct FileInfo {
    filename: PathBuf,
    hash: u64,
}

const HAMMING_DISTANCE: u8 = 3;

pub fn start(hasher_done_recv: Receiver<()>, status_sndr: Sender<StatusMgrMsg>) -> Receiver<()> {
    let (done_sender, done_receiver) = crossbeam_channel::bounded(1);

    thread::spawn(move || {
        // Wait for the hasher to be done.
        first_or_default(hasher_done_recv);

        let (filenames_sndr, filenames_recv) = crossbeam_channel::bounded(1);
        status_sndr
            .send(StatusMgrMsg::FilesRequest(filenames_sndr))
            .unwrap();

        let filenames = first_or_default(filenames_recv);
        let mut map = DupFinderMap::new(|id| id.a_hash);
        let mut fns_with_hashes = Vec::<(PathBuf, u64)>::new();
        for filename in &filenames {
            let (id_sndr, id_recv) = crossbeam_channel::bounded(1);
            let afn = Arc::new(filename.clone());
            status_sndr.send(GetImageData(afn, id_sndr)).unwrap();
            let image_data = first_or_default(id_recv).unwrap_or_default();
            if let Some(hsh) = map.add_image(filename, &image_data) {
                fns_with_hashes.push((filename.to_path_buf(), hsh));
            }
        }

        for (filename, hsh) in fns_with_hashes {
            let near_images = map.find_near_images(hsh, HAMMING_DISTANCE);
            if near_images.len() > 0 {
                println!("Found {} near images.", near_images.len());
                near_images.iter().for_each(|path| {
                    println!("  {}", path.display());
                });
            }
        }

        done_sender.send(()).unwrap_or_default()
    });

    done_receiver
}
