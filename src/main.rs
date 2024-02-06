use std::env;
use std::fs::File;
use std::cmp;
use std::io::{BufRead, BufReader};
use image::{GenericImageView, ImageBuffer, RgbImage, imageops};

const BUFFER_SIZE: usize = 3000;
const COLOR_SPACE: usize = 3;

struct ImageConverter {
    path: String
}

impl ImageConverter {
    pub fn new(path: &str) -> ImageConverter {
        ImageConverter { path: String::from(path) }
    }

    pub fn to_jpg(&self) -> std::io::Result<()> {
        let f = File::open(&self.path)?;
        let metadata = f.metadata()?;
        let file_size = metadata.len();
        let pixels = file_size.div_ceil(COLOR_SPACE as u64);
        let d = (pixels as f64).sqrt().ceil() as u32;

        let mut reader = BufReader::with_capacity(BUFFER_SIZE, f);
        let mut reads = 0;
        let mut imgbuf = ImageBuffer::new(d, d);
        let mut n = 0;

        loop {
            let buffer = reader.fill_buf()?;
            let length = buffer.len();
            if length == 0 {
                break;
            }

            let mut start = 0;

            loop {
                let m = cmp::min(length, start + COLOR_SPACE);
                let slice = &buffer[start..m];
                // NOTE: I'm missing the last buffer ....
                //println!("{:?}", slice);

                if slice.len() < COLOR_SPACE {
                    break;
                }

                let x = n % d;
                let y = n / d;
                let pixel = imgbuf.get_pixel_mut(x, y);
                *pixel = image::Rgb([slice[0], slice[1], slice[2]]);

                n += 1;
                start += COLOR_SPACE
            }

            reader.consume(length);
            reads += 1;
        }
        println!("{}", reads);
        imgbuf.save("test.jpg").unwrap();
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        panic!("Invalid number of arguments");
    }
    let file = ImageConverter::new(&args[1]);
    return file.to_jpg();
}

#[test]
fn to_image_test() {
    let file = ImageConverter::new("files/test.jpg");
    file.to_jpg();
}

#[test]
fn from_image_test() {

}
