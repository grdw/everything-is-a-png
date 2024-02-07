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

// TODO: Figure out if you could (ab)use the alpha channel
// of the PNG to read 4 bytes of the top.
impl ImageConverter {
    pub fn new(path: &str) -> ImageConverter {
        ImageConverter { path: String::from(path) }
    }

    // TODO: Make your buffer 1024 bytes, and not 3 bytes
    // TODO: In the final blurb, trim off the last 0's
    pub fn from_image(&self) -> std::io::Result<()> {
        let n = self.path.len();
        let t = String::from(&self.path[0..n-4]);
        let mut file = File::create(format!("unwrapped.{}", t))?;
        let img = image::open(&self.path).unwrap();

        let mut buffer = [0; BUFFER_SIZE];
        let mut n = 0;
        for (_x, _y, pixel) in img.pixels() {
            let p = &pixel.0[0..COLOR_SPACE];

            for (k, j) in (n..n+COLOR_SPACE).enumerate() {
                buffer[j] = p[k]
            }

            n += COLOR_SPACE;

            if n % BUFFER_SIZE == 0 {
                file.write_all(&buffer)?;
                buffer = [0; BUFFER_SIZE];
                n = 0;
            }
        }
        Ok(())
    }

    pub fn to_image(&self) -> std::io::Result<()> {
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
                let y = n / d;
                let pixel = imgbuf.get_pixel_mut(x, y);
                let mut buf = [0; COLOR_SPACE];

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
        "img" => file.to_image(),
        "unimg" => file.from_image(),
        _ => panic!("Invalid first argument")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn to_image_test() {
        let file = ImageConverter::new("cat.jpg");
        let _ = file.to_image();
        let file = ImageConverter::new("cat.jpg.png");
        let _ = file.from_image();
        // Test if the file size is the same and do some md5 test

        // Clean-up:
        fs::remove_file("cat.jpg.png").unwrap();
        fs::remove_file("unwrapped.cat.jpg").unwrap();
    }
}
