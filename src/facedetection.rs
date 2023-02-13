// facedetection module

use opencv::core::{Vector, Rect, Size, Scalar};
use opencv::imgproc::{rectangle, LINE_8};
use opencv::imgcodecs::{imread, imwrite, IMREAD_GRAYSCALE, IMREAD_COLOR};
use opencv::objdetect::CascadeClassifier;
use opencv::prelude::CascadeClassifierTrait;

const LBP_CLASSIFIER_FILEPATH : &'static str = "lbpcascade_frontalface_improved.xml";

pub fn find_all_faces(image_filepath : &str, classifier_filepath : Option<&str>, verbose : bool) -> Result<Vector<Rect>, ()> {

    let classifier_filepath : &str = match classifier_filepath {
        Some(filepath) => filepath,
        None => LBP_CLASSIFIER_FILEPATH,
    };

    let greyscaled_face_image = match imread(image_filepath, IMREAD_GRAYSCALE) {
        Ok(img) => img,
        Err(_) => return Err(()),
    };

    let mut cascade_detector = match CascadeClassifier::new(classifier_filepath) {
        Ok(cascade) => cascade,
        Err(_) => return Err(()),
    };

    // found these by trial an error, might try and up the scale factor if it's too slow
    let mut faces_detected : Vector<Rect> = Vector::new();
    let scale_factor : f64 = 1.25;
    let min_neighbours : i32 = 5;
    let flags : i32 = 0;
    let min_size : Size = Size::new(0, 0);
    let max_size : Size = Size::new(0, 0);

    if verbose {
        println!("running facial detection with the following settings :\n   * scale_factor : {}\n   * min_neighbours : {}\n   * flags : {}\n   * min_size : {}\n   * max_size : {}",
            scale_factor, min_neighbours, flags, min_size.area(), max_size.area()
        );
    }

    match cascade_detector.detect_multi_scale(&greyscaled_face_image, &mut faces_detected, scale_factor, min_neighbours, flags, min_size, max_size) {
        Ok(_) => (),
        Err(_) => return Err(()),
    }

    if verbose {
        println!("number of faces detected : {}", faces_detected.len());
    }
    
    return Ok(faces_detected);
}

pub fn draw_rectangle(image_filepath : &str, output_filepath : &str, faces : &Vector<Rect>) -> Result<(), ()> {

    let mut image = match imread(image_filepath, IMREAD_COLOR) {
        Ok(img) => img,
        Err(_) => panic!("could not read the specified image"),
    };

    for face in faces.iter() {
        let thickness : i32 = calculate_appropriate_rect_thickness(face.area());
        match rectangle(&mut image, face, Scalar::new(0.0, 0.0, 255.0, 0.0), thickness,  LINE_8, 0) {
            Ok(_) => (),
            Err(_) => return Err(()),
        }
    }
    let params : Vector<i32> = Vector::new();
    match imwrite(output_filepath, &image, &params) {
        // not sure ignoring the "false" bool result is wise but I didn't see anything about it on the openCV docs
        Ok(_) => return Ok(()),
        Err(_) => return Err(()),
    }
}

fn calculate_appropriate_rect_thickness(area : i32) -> i32 {
    let area : f64 = f64::from(area);
    let side : f64 = area.sqrt();
    let mut divided_side : f64 = side / 45.0;
    divided_side = divided_side.ceil();
    return divided_side as i32;
}



