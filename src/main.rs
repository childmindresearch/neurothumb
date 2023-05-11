use std::env;

use image::GrayImage;
use ndarray::{Array2, Ix2};
use nifti::{IntoNdArray, NiftiHeader, NiftiObject, NiftiVolume, ReaderStreamedOptions};

/// Scale x and y relative so that the bigger one will be max_val.
fn scale_relative(x: u32, y: u32, max_val: u32) -> (u32, u32) {
    let (fx, fy, fmax_val) = (x as f32, y as f32, max_val as f32);

    let fxy_max = std::cmp::max(x, y) as f32;

    let out_x = std::cmp::min(((fx / fxy_max) * fmax_val) as u32, max_val);
    let out_y = std::cmp::min(((fy / fxy_max) * fmax_val) as u32, max_val);

    return (out_x, out_y);
}

fn normalize_u8(data: &Array2<f32>) -> Array2<u8> {
    let imin = *data
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
    let imax = *data
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();

    return (((data - imin) / (imax - imin)) * 255.).mapv(|v| v as u8);
}

fn make_image_gray(data: Array2<u8>) -> GrayImage {
    let width = data.shape()[0] as u32;
    let height = data.shape()[1] as u32;
    return GrayImage::from_raw(width, height, data.into_raw_vec()).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let arg_input = &args[1];
    let arg_output = &args[2];
    let arg_size: u32 = args[3].parse().unwrap();

    let obj = ReaderStreamedOptions::new()
        .read_file_rank(arg_input, 2)
        .expect("nifti2 not supported yet");

    let mut obj_volume = obj.into_volume();

    let data: Array2<f32> = match obj_volume.dimensionality() {
        2 => obj_volume
            .read_slice()
            .unwrap()
            .into_ndarray::<f32>()
            .unwrap()
            .into_dimensionality::<Ix2>()
            .unwrap(),
        d if d >= 3 => {
            let mid_slice = obj_volume.dim()[2] / 2;

            obj_volume
                .nth(mid_slice as usize)
                .unwrap()
                .unwrap()
                .into_ndarray::<f32>()
                .unwrap()
                .into_dimensionality::<Ix2>()
                .unwrap()
        }
        _ => panic!("Unsupported dimensionality"),
    };

    let data_norm = normalize_u8(&data);

    let width = data.shape()[0] as u32;
    let height = data.shape()[1] as u32;

    let (out_width, out_height) = scale_relative(width as u32, height as u32, arg_size);

    let thumb = make_image_gray(data_norm);

    let thumb_sized = image::imageops::resize(
        &thumb,
        out_width,
        out_height,
        image::imageops::FilterType::Lanczos3,
    );

    thumb_sized.save(arg_output).unwrap();
}
