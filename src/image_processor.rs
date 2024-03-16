use std::fs::File;
use std::path::Path;

use image::{DynamicImage, ImageOutputFormat};
use image::imageops::FilterType;

use crate::Context;
use crate::util::nanoid;


pub enum ResizeMode {
    /// Perform a crop to a rectangle with the target aspect ratio (if needed),
    /// then resize to a maximum width.
    /// Aspect ratio is width / height, e.g. 16/9 = 1.7777777
    CoverRectangle { aspect: f32, max_width: u32 }
}

pub struct ImageProcessor {
    pub source_height: u32,
    source_image: DynamicImage,
    pub source_width: u32
}

impl ImageProcessor {
    fn export(
        &self,
        context: &Context,
        resized_image: &DynamicImage
    ) -> (String, (u32, u32)) {
        let result_dimensions = (resized_image.width(), resized_image.height());

        let output_filename = format!("{}.jpg", nanoid());
        let output_path = context.cache_dir.join(&output_filename);

        let mut output_file = File::create(output_path).unwrap();

        resized_image.write_to(&mut output_file, ImageOutputFormat::Jpeg(80)).unwrap();

        (output_filename, result_dimensions)
    }

    fn resize_and_export(
        &self,
        context: &Context,
        max_width: u32,
        cropped_image: &DynamicImage
    ) -> (String, (u32, u32)) {
        let cropped_width = cropped_image.width();
        
        if cropped_width > max_width {
            let resize_factor = max_width as f32 / cropped_width as f32;
            let new_width = (self.source_width as f32 * resize_factor) as u32;
            let new_height = (self.source_height as f32 * resize_factor) as u32;
            let resized_image = cropped_image.resize(new_width, new_height, FilterType::Lanczos3);
            self.export(context, &resized_image)
        } else {
            self.export(context, cropped_image)
        }
    }

    pub fn open(path: &Path) -> ImageProcessor {
        let source_image = image::open(path).unwrap();
        let source_height = source_image.height();
        let source_width = source_image.width();

        ImageProcessor {
            source_height,
            source_image,
            source_width
        }
    }

    pub fn resize(
        &self,
        context: &Context,
        resize_mode: ResizeMode
    ) -> (String, (u32, u32)) {
        match resize_mode {
            ResizeMode::CoverRectangle { aspect, max_width } => {
                let found_aspect = self.source_width as f32 / self.source_height as f32;
                if found_aspect < aspect {
                    // too tall, reduce height
                    let new_height = (self.source_width as f32 / aspect).floor() as u32;
                    let y = (self.source_height - new_height) / 2;
                    let cropped_image = self.source_image.crop_imm(0, y, self.source_width, new_height);
                    self.resize_and_export(context, max_width, &cropped_image)
                } else if found_aspect > aspect {
                    // too wide, reduce width
                    let new_width = (aspect * self.source_height as f32).floor() as u32;
                    let x = (self.source_width - new_width) / 2;
                    let cropped_image = self.source_image.crop_imm(x, 0, new_width, self.source_height);
                    self.resize_and_export(context, max_width, &cropped_image)
                } else {
                    self.resize_and_export(context, max_width, &self.source_image)
                }
            }
        }
    }
}