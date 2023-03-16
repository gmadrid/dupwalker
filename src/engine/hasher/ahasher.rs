use image::imageops::FilterType;
use image::DynamicImage;

pub fn ahash(image: &DynamicImage) -> u64 {
    let reduced = image.resize_exact(8, 8, FilterType::Gaussian).to_luma16();
    let avg = (reduced.pixels().map(|p| p[0] as u32).sum::<u32>() / (8 * 8)) as u16;
    let mut hash = 0u64;

    for row in 0..8 {
        for col in 0..8 {
            hash <<= 1;
            if reduced.get_pixel(col, row)[0] > avg {
                hash += 1;
            }
        }
    }

    hash
}
