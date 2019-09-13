extern crate packed_simd;

use packed_simd::*;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

fn main() {
    const IMG_WIDTH: i32 = 8192;
    const IMG_HEIGHT: i32 = 8192;
    let max_iterations = 32;
    let cmin_x: f32 = -2.0;
    let cmax_x: f32 = 1.0;
    let cmin_y: f32 = -1.5;
    let cmax_y: f32 = 1.5;

    let scale_x = (cmax_x - cmin_x) / (IMG_WIDTH as f32);
    let scale_y = (cmax_y - cmin_y) / (IMG_HEIGHT as f32);
    let iter_scale = 255 / max_iterations;

    let mut image: Vec<u8> = vec![0; (IMG_WIDTH * IMG_HEIGHT * 3) as usize];

    for y in 0..IMG_HEIGHT {
        for x in (0..IMG_WIDTH).step_by(8) {
            {
                let xf = x as f32;
                let vx: f32x8 = f32x8::new(
                    xf,
                    xf + 1.0,
                    xf + 2.0,
                    xf + 3.0,
                    xf + 4.0,
                    xf + 5.0,
                    xf + 6.0,
                    xf + 7.0,
                );
                let cx = cmin_x + (vx * scale_x);
                let cy = cmax_y - (f32x8::splat(y as f32) * scale_y);
                let mut zx = cx;
                let mut zy = cy;

                let mut ret = i32x8::splat(1);
                let mask = m32x8::splat(true);

                for _ in 0..max_iterations {
                    let mut zx2 = zx * zx;
                    let mut zy2 = zy * zy;
                    let zxzy = zx * zy;
                    zx = mask.select(zx2 - zy2 + cx, zx);
                    zy = mask.select(zxzy + zxzy + cy, zy);

                    zx2 = zx * zx;
                    zy2 = zy * zy;
                    let mask = (zx2 + zy2).le(f32x8::splat(4.0));
                    ret = mask.select(ret + 1, ret);

                    if mask.none() {
                        break;
                    }
                }

                for i in 0i32..8 {
                    let j = (255 - (ret.extract(i as usize) * iter_scale) % 255) as u8;
                    let index: usize = ((y * IMG_WIDTH + x + i) * 3) as usize;
                    image[index] = (255 * x / IMG_HEIGHT) as u8;
                    image[index + 1] = j;
                    image[index + 2] = (255 * y / IMG_WIDTH) as u8;
                }
            }
        }
    }

    let path = Path::new(r"image.png");
    let file = File::create(path).unwrap();
    let ref mut bufw = BufWriter::new(file);
    
    let mut encoder = png::Encoder::new(bufw, IMG_WIDTH as u32, IMG_WIDTH as u32);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&image).unwrap();
}
