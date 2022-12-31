extern crate image;

use image::{GenericImageView, Pixel};
use image::open;
use image::imageops;

pub struct Video {
    width: u32,
    height: u32,
    frames: Vec<Vec<f32>>,
}

impl Video {
    pub fn new(path: &str, width: u32, height: u32) -> Self {
        let mut index = 1; // ffmpeg starts at 1
        let mut frames = vec![];
        loop {
            let path = format!("{}/f{:0>4}.png", path, index);
            let img = match open(path) {
                Ok(img) => img,
                Err(_) => break,
            };
            let img = img.resize(width, height, imageops::FilterType::Nearest);
            let mut frame = vec![];
            for pixel in img.pixels() {
                let mut i = pixel.2.channels().iter().map(|a| *a as f32);
                let (r, g, b) = (i.next().unwrap(), i.next().unwrap(), i.next().unwrap());
                let p = (r + g + b) as f32 / (255.0 * 3.0);
                frame.push(p);
            }
            frames.push(frame);
            index += 1;
        }
        Self {
            width,
            height,
            frames,
        }
    }

    pub fn get_frame(&self, index: usize) -> &[f32] {
        &self.frames[index]
    }

    pub fn get_frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn get_pixel_value(&self, index: usize, x: u32, y: u32) -> f32 {
        self.frames[index][(y * self.width + x) as usize]
    }

    pub fn get_rc_formatted_frame(&self, index: usize) -> Vec<Vec<f32>> {
        let mut frame = vec![];
        for y in 0..self.height {
            let mut row = vec![];
            for x in 0..self.width {
                row.push(self.get_pixel_value(index, x, y));
            }
            frame.push(row);
        }
        frame
    }

    pub fn get_rc_formatted_frame_string(&self, index: usize) -> String {
        let mut frame = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                frame.push_str(&format!("{:.4}, ", self.get_pixel_value(index, x, y)));
            }
            frame.push_str(&format!("\n"));
        }
        frame
    }

    pub fn get_xy_formatted_frame(&self, index: usize) -> Vec<Vec<f32>> {
        let mut frame = vec![];
        for x in 0..self.width {
            let mut row = vec![];
            for y in 0..self.height {
                row.push(self.get_pixel_value(index, x, y));
            }
            frame.push(row);
        }
        frame
    }

    pub fn frame_index_from_time(&self, time: f32, fps: f32) -> usize {
        (time * fps) as usize
    }
}