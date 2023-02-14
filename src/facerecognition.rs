// facerecognition module

use opencv::face::LBPHFaceRecognizer;
use opencv::face::LBPHFaceRecognizerConst;

struct FRparams {
    radius : i32,
    neighbors : i32,
    grid_x : i32,
    grid_y : i32,
    threshold : f64,
}

impl FRparams {
    pub fn new(radius : i32, neighbors : i32, grid_x : i32, grid_y : i32, threshold : f64) -> FRparams {
        return FRparams {
            radius : radius,
            neighbors : neighbors,
            grid_x : grid_x,
            grid_y : grid_y,
            threshold : threshold
        };
    }
    pub fn default() -> FRparams {
        return FRparams {
            radius : 1,
            neighbors : 8,
            grid_x : 8,
            grid_y : 8,
            threshold : f64::MAX,
        };
    }
}


pub fn recognize_face() {
    let params : FRparams = FRparams::default();
    let face_recognizer = <dyn LBPHFaceRecognizer>::create(params.radius, params.neighbors, params.grid_x, params.grid_y, params.threshold);
}