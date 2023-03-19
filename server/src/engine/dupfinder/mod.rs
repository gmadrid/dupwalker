use crate::engine::status_mgr::ImageData;
use std::cmp::{max, min};
use std::path::{Path, PathBuf};

struct DupFinderMap<G: Fn(&ImageData) -> u64> {
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

impl<G: Fn(&ImageData) -> u64> DupFinderMap<G> {
    fn new(hash_getter: G) -> DupFinderMap<G> {
        DupFinderMap {
            hash_getter,
            lookup: [(); 65].map(|_| vec![]),
        }
    }

    pub fn add_image(&mut self, path: &Path, image_data: &ImageData) {
        let hash = (self.hash_getter)(image_data);
        let bit_count = count_bits(hash);
        let v = &mut self.lookup[bit_count as usize];
        v.push(FileInfo {
            filename: path.to_path_buf(),
            hash,
        });
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
