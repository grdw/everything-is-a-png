struct ImageConverter {
    path: String
}

impl ImageConverter {
    pub fn new(path: &'static str) -> ImageConverter {
        ImageConverter { path: String::from(path) }
    }
}

fn main() {
    println!("Hello, world!");
}

#[test]
fn to_image_test() {
    let file = ImageConverter::new("files/test.jpg");
    file.toJPG();
}

#[test]
fn from_image_test() {

}
