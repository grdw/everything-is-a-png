use std::env;
use std::fs::File;
use std::cmp;
use std::io::{BufRead, BufReader, Write};
use image::{GenericImageView, ImageBuffer};

const BUFFER_SIZE: usize = 3000;
const COLOR_SPACE: usize = 3;

struct ImageConverter {
    path: String
}

impl ImageConverter {
    pub fn new(path: &str) -> ImageConverter {
        ImageConverter { path: String::from(path) }
    }

    pub fn from_jpg(&self) -> std::io::Result<()> {
        let n = self.path.len();
        let t = String::from(&self.path[0..n-4]);
        let mut file = File::create(format!("unwrapped.{}", t))?;
        let img = image::open(&self.path).unwrap();
        for (_x, _y, pixel) in img.pixels() {
            let buf = [pixel.0[0], pixel.0[1], pixel.0[2]];
            file.write_all(&buf)?;
        }
        Ok(())
    }

    pub fn to_jpg(&self) -> std::io::Result<()> {
        let f = File::open(&self.path)?;
        let metadata = f.metadata()?;
        let file_size = metadata.len();
        let pixels = file_size.div_ceil(COLOR_SPACE as u64);
        let d = (pixels as f64).sqrt().ceil() as u32;

        let mut reader = BufReader::with_capacity(BUFFER_SIZE, f);
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
                if start > length {
                    break;
                }

                let m = cmp::min(length, start + COLOR_SPACE);
                let slice = &buffer[start..m];
                if slice.len() == 0 {
                    break;
                }

                let x = n % d;
                let mut y = n / d;


                let pixel = imgbuf.get_pixel_mut(x, y);
                let mut buf = [0, 0, 0];

                for j in 0..slice.len() {
                    buf[j] = slice[j];
                }

                *pixel = image::Rgb(buf);

                n += 1;
                start += COLOR_SPACE
            }

            reader.consume(length);
        }
        imgbuf.save(format!("{}.png", self.path)).unwrap();
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Invalid number of arguments");
    }
    let file = ImageConverter::new(&args[2]);
    return match args[1].as_str() {
        "jpg" => file.to_jpg(),
        "unjpg" => file.from_jpg(),
        _ => panic!("Invalid first argument")
    }
}

#[test]
fn to_image_test() {
    let file = ImageConverter::new("cat.jpg");
    file.to_jpg();
}

#[test]
fn from_image_test() {

}
