// siberia version 0.1
// written by Oether

mod facedetection;
mod facerecognition;

fn main() {
    let faces = facedetection::find_all_faces("testing/img1.jpg", None, true).unwrap();
    facedetection::draw_rectangle("testing/img1.jpg", "output.jpg", &faces);
}