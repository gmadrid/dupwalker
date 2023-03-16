use image::imageops::FilterType;
use image::DynamicImage;

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
