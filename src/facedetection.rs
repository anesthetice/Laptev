

use std::time::{Instant, Duration};
use opencv::core::{Mat, Vector, Rect, Size, Scalar};
use opencv::imgproc::{cvt_color, COLOR_BGR2RGB, rectangle, LINE_8};
use opencv::imgcodecs::{imread, imwrite, IMREAD_GRAYSCALE, IMREAD_COLOR};
use opencv::highgui::{imshow, wait_key, destroy_all_windows};
use opencv::objdetect::CascadeClassifier;
use opencv::prelude::CascadeClassifierTrait;


// currently testing and benchmarking
pub fn find_all_faces(image_filepath : &str, classifier_filepath : &str) -> Vector<Rect> {

    let greyscaled_face_image = match imread(image_filepath, IMREAD_GRAYSCALE) {
        Ok(img) => img,
        Err(_) => panic!("could not read the specified image"),
    };
    let mut colored_face_image = match imread(image_filepath, IMREAD_COLOR) {
        Ok(img) => img,
        Err(_) => panic!("could not read the specified image"),
    };

    let mut cascade_detector = CascadeClassifier::new(classifier_filepath).unwrap();

    let mut faces_detected : Vector<Rect> = Vector::new();
    let scale_factor : f64 = 1.2;
    let min_neighbours : i32 = 5;
    let flags : i32 = 0;
    let min_size : Size = Size::new(0, 0);
    let max_size : Size = Size::new(0, 0);

    println!("running the facial detection with the following settings :\n   *scale_factor : {}\n   *min_neighbours : {}\n   *flags : {}\n   *min_size : {}\n   *max_size : {}",
        scale_factor, min_neighbours, flags, min_size.area(), max_size.area()
    );

    let now : Instant  = Instant::now();

    cascade_detector.detect_multi_scale(&greyscaled_face_image, &mut faces_detected, scale_factor, min_neighbours, flags, min_size, max_size).unwrap();

    println!("detected {} faces in {} ms", faces_detected.len(), now.elapsed().as_millis());
    

    let mut fdclone = faces_detected.clone();
    for face in fdclone {
        rectangle(&mut colored_face_image, face, Scalar::new(0.0, 0.0, 255.0, 0.0), 7,  LINE_8, 0).unwrap();
    }
    let params : Vector<i32> = Vector::new();
    imwrite("output.jpg", &colored_face_image, &params).unwrap();
    return faces_detected;

}


