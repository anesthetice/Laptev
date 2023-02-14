// siberia version 0.1
// written by Oether

mod facedetection;
mod facerecognition;

fn main() {
    let faces = facedetection::find_all_faces("testing/img1.jpg", Some("haarcascade_frontalface_alt.xml"), None, true).unwrap();
    facedetection::draw_rectangle("testing/img1.jpg", "output1.jpg", &faces);

    let faces = facedetection::find_all_faces("testing/img2.jpg", Some("haarcascade_frontalface_alt.xml"), None, true).unwrap();
    facedetection::draw_rectangle("testing/img2.jpg", "output2.jpg", &faces);

    let faces = facedetection::find_all_faces("testing/img3.jpg", Some("haarcascade_frontalface_alt.xml"), None, true).unwrap();
    facedetection::draw_rectangle("testing/img3.jpg", "output3.jpg", &faces);
}