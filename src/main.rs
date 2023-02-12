// siberihttp://vision.ucsd.edu/content/yale-face-databasea version 0.1
// written by Oether

//use opencv::prelude::*;
use rustface::{Detector, FaceInfo, ImageData};



use image::{DynamicImage, GrayImage, Rgb};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;

const OUTPUT_FILE: &str = "test.png";

fn main() {

    let mut detector = rustface::create_detector("/home/ether/Shared/Warehouse/Rust/siberia/seeta_fd_frontal_v1.0.bin").unwrap();
    detector.set_min_face_size(20);
    detector.set_score_thresh(0.95);
    detector.set_pyramid_scale_factor(0.8);
    detector.set_slide_window_step(4, 4);

    let image: DynamicImage = image::open("/home/ether/Shared/Warehouse/Rust/siberia/twofaces.png").unwrap();

    let mut rgb = image.to_rgb8();
    let faces = detect_faces(&mut *detector, &image.to_luma8());

    for face in faces {
        let bbox = face.bbox();
        let rect = Rect::at(bbox.x(), bbox.y()).of_size(bbox.width(), bbox.height());

        draw_hollow_rect_mut(&mut rgb, rect, Rgb([255, 0, 0]));
    }

    match rgb.save(OUTPUT_FILE) {
        Ok(_) => println!("Saved result to {}", OUTPUT_FILE),
        Err(message) => println!("Failed to save result to a file. Reason: {}", message),
    }
}

fn detect_faces(detector: &mut dyn Detector, gray: &GrayImage) -> Vec<FaceInfo> {
    let (width, height) = gray.dimensions();
    let mut image = ImageData::new(gray, width, height);
    let faces = detector.detect(&mut image);
    println!(
        "Found {} faces",
        faces.len(),
    );
    faces
}