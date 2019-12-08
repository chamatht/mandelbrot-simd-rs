extern crate packed_simd;

use packed_simd::*;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

fn main() {
    const IMG_WIDTH: i32 = 1024;
    const IMG_HEIGHT: i32 = 1024;
    let max_iterations = 32;
    let cmin_r: f32 = -2.0;
    let cmax_r: f32 = 1.0;
    let cmin_i: f32 = -1.5;
    let cmax_i: f32 = 1.5;

    let scale_x = (cmax_r - cmin_r) / (IMG_WIDTH as f32);
    let scale_y = (cmax_i - cmin_i) / (IMG_HEIGHT as f32);
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
                let c_r = cmin_r + (vx * scale_x);
                let c_i = cmax_i - (f32x8::splat(y as f32) * scale_y);
                let mut z_r = c_r;
                let mut z_i = c_i;

                let mut ret = i32x8::splat(1);
                let mask = m32x8::splat(true);

                for _ in 0..max_iterations {
                    let mut z_r2 = z_r * z_r;
                    let mut z_i2 = z_i * z_i;
                    let zrzi = z_r *z_i;
                    z_r = mask.select(z_r2 - z_i2 + c_r, z_r);
                    z_i = mask.select(zrzi + zrzi + c_i, z_i);

                    z_r2 = z_r * z_r;
                    z_i2 = z_i * z_i;
                    let mask = (z_r2 +z_i2).le(f32x8::splat(4.0));
                    ret = mask.select(ret + 1, ret);

                    if mask.none() {
                        break;
                    }
                }

                for c in 0i32..8 {
                    let color = (255 - (ret.extract(c as usize) * iter_scale) % 255) as u8;
                    let index = ((y * IMG_WIDTH + x + c) * 3) as usize;
                    image[index] = (255 * x / IMG_HEIGHT) as u8;
                    image[index + 1] = color;
                    image[index + 2] = (255 * y / IMG_WIDTH) as u8;
                }
            }
        }
    }

    let path = Path::new(r"image.png");
    let file = File::create(path).unwrap();
    let bufw = &mut BufWriter::new(file);
    
    let mut encoder = png::Encoder::new(bufw, IMG_WIDTH as u32, IMG_WIDTH as u32);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_color(png::ColorType::RGB);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&image).unwrap();
}

