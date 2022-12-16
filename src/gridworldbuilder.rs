use egui::Image;
use image::{GenericImage, GenericImageView, ImageBuffer, RgbImage,RgbaImage, Pixel, Rgba, Pixels, buffer::PixelsMut,PixelWithColorType,GrayAlphaImage};
use itertools::{Itertools,Either};
use core::option::Iter;
use ndarray::{Zip,Array2,arr2,s,Axis,Array3, Array};


pub fn construct_grid_img(w : usize,h:usize,cells_hori : usize, cells_vert : usize,border_w : usize) -> GrayAlphaImage{
    let mut img = Array3::<u8>::zeros((h,w,2));
    let tile_len = h.checked_div(cells_hori).expect("could not divide") as usize; 
    let vec = Array::from_vec(vec![0,255u8]);
    for cell_nr in (1usize..cells_hori){
        let to_black_out = (tile_len * cell_nr-border_w.. tile_len * cell_nr + border_w + 1);
        img.slice_mut(s![to_black_out,..,..]).assign(&vec);
    }
    let tile_len = w.checked_div(cells_vert).expect("could not divide") as usize; 
    for cell_nr in (1usize..cells_vert){
        let to_black_out = (tile_len * cell_nr-border_w.. tile_len * cell_nr + border_w + 1);
        img.slice_mut(s![..,to_black_out,..]).assign(&vec);
    }
    let buffer = img.into_raw_vec();
    let img = GrayAlphaImage::from_raw(w as u32,h as u32,buffer).expect("error during loading grid world buffer as img");
    //img.save("output.png");
    return img;
}


fn array_to_image(arr: Array3<u8>) -> RgbImage {
    assert!(arr.is_standard_layout());
    let (height, width, _) = arr.dim();
    let raw = arr.into_raw_vec();

    RgbImage::from_raw(width as u32, height as u32, raw)
        .expect("container should have the right size for the image dimensions")
}


