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
// TODO: How do you dunk out the filesize into image space?
// Or perhaps: how do you indicate an "END" like 50 white pixels or
// something dumb like that?
// TODO: What's the max file size of a PNG? How much gunk can you
// dunk?
impl ImageConverter {
    pub fn new(path: &str) -> ImageConverter {
        ImageConverter { path: String::from(path) }
    }

    pub fn from_image(&self) -> std::io::Result<()> {
        let n = self.path.len();
        let t = String::from(&self.path[0..n-4]);
        let mut file = File::create(format!("unwrapped.{}", t))?;
        let img = image::open(&self.path).unwrap();
        let (width, _) = img.dimensions();

        let mut buffer = vec![];
        let mut n = 0;
        for (x, y, pixel) in img.pixels() {
            for (k, j) in (n..n+COLOR_SPACE).enumerate() {
                buffer.insert(j, pixel.0[k])
            }

            n += COLOR_SPACE;

            if n % BUFFER_SIZE == 0 {
                // NOTE: Ugly hack incoming
                // Ugh ... I should probably know the original
                // filesize ...
                if y + 1 == width {
                    let mut fixed_buffer = vec![];
                    for i in buffer {
                        if i == 0 { continue }
                        fixed_buffer.insert(0, i)
                    }
                    buffer = fixed_buffer;
                }
                file.write_all(&buffer)?;
                buffer = vec![];
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

    fn file_check(path: &str) -> (u64, String) {
        let f = File::open(path).unwrap();
        let metadata = f.metadata().unwrap();
        let file_size = metadata.len();

        let mut reader = BufReader::with_capacity(BUFFER_SIZE, f);

        loop {
            let buffer = reader.fill_buf().unwrap();
            let length = buffer.len();
            if length == 0 {
                break;
            }

            reader.consume(length);
        }

        (file_size, String::from(""))
    }

    #[test]
    fn to_image_test() {
        let files = vec!["cat.jpg", "hello-world.txt"];
        for f in files {
            let file = ImageConverter::new(f);
            let conv_name = format!("{}.png", f);
            let reconv_name = format!("unwrapped.{}", f);
            let _ = file.to_image();
            let file = ImageConverter::new(&conv_name);
            let _ = file.from_image();
            // Test if the file size is the same and do some md5 test
            let (s1, md51) = file_check(f);
            let (s2, md52) = file_check(&reconv_name);

            assert_eq!(s1, s2);
            assert_eq!(md51, md52);

            // Clean-up:
            fs::remove_file(&conv_name).unwrap();
            fs::remove_file(&reconv_name).unwrap();
        }
    }
}
