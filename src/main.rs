// siberia version 0.1
// written by Oether

mod facedetection;

fn main() {
    let class_filepath_0 : &str = "/home/ether/Shared/Warehouse/Rust/siberia/lbpcascade_frontalface_improved.xml";
    let image_filepath : &str = "/home/ether/Shared/Warehouse/Rust/siberia/testing/doubleface.jpg";

    facedetection::find_all_faces(image_filepath, class_filepath_0);
}