// facedetection module

use opencv::core::{Vector, Rect, Size, Scalar};
use opencv::imgproc::{rectangle, LINE_8};
use opencv::imgcodecs::{imread, imwrite, IMREAD_GRAYSCALE, IMREAD_COLOR};
use opencv::objdetect::CascadeClassifier;
use opencv::prelude::CascadeClassifierTrait;

const LBP_CLASSIFIER_FILEPATH : &'static str = "lbpcascade_frontalface_improved.xml";

pub struct DCMparams {
    scale_factor : f64,
    min_neighbours : i32,
    flags : i32,
    min_size : Size,
    max_size  : Size,
}

impl DCMparams {
    pub fn new(scale_factor : f64, min_neihbours : i32, flags : i32, min_size : (i32, i32), max_size : (i32, i32)) -> DCMparams {
        // size is (width, height)
        return DCMparams {
            scale_factor : scale_factor,
            min_neighbours : min_neihbours,
            flags : flags,
            min_size : Size::new(min_size.0, min_size.1),
            max_size : Size::new(max_size.0, max_size.1)
        }
    }
    pub fn default() -> DCMparams {
        // mostly trial and error to find this, subject to change
        return DCMparams {
            scale_factor : 1.25,
            min_neighbours : 5,
            flags : 0,
            min_size : Size::new(0, 0),
            max_size : Size::new(0, 0)
        }
    }
}

pub fn find_all_faces(image_filepath : &str, classifier_filepath : Option<&str>, custom_detector_settings : Option<DCMparams>,verbose : bool) -> Result<Vector<Rect>, ()> {

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

    let params : DCMparams = match custom_detector_settings {
        Some(params) => params,
        None => DCMparams::default(),
    };
    let mut faces_detected : Vector<Rect> = Vector::new();

    if verbose {
        println!("running facial detection with the following settings :\n   * scale_factor : {}\n   * min_neighbours : {}\n   * flags : {}\n   * min_size : {}\n   * max_size : {}",
            params.scale_factor, params.min_neighbours, params.flags, params.min_size.area(), params.max_size.area()
        );
    }

    match cascade_detector.detect_multi_scale(&greyscaled_face_image, &mut faces_detected, params.scale_factor, params.min_neighbours, params.flags, params.min_size, params.max_size) {
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



