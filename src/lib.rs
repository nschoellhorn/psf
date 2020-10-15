pub struct Font {
    data: Vec<Vec<u8>>,
    width: usize,
    height: usize,
    byte_width: usize,
}

#[derive(Debug)]
pub struct Vec2d<T> {
    d: Vec<T>,
    height: usize,
    width: usize,
}

impl<T> Vec2d<T> {
    pub fn data(&self) -> &Vec<T> {
        &self.d
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

#[derive(Debug)]
pub enum Error {
    Unknown,
    FileNotFound,
    FileIo,
    InvalidFontFormat,
}

impl std::convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        match e {
            _ => Error::FileIo,
        }
    }
}

impl Font {
    pub fn new_from_str(path: &str) -> Result<Font, Error> {
        Font::new(&std::path::Path::new(path))
    }

    pub fn new(path: &std::path::Path) -> Result<Font, Error> {
        if !path.exists() && !path.is_file() {
            return Err(Error::FileNotFound);
        }

        let filename = path.file_name();
        if filename.is_none() {
            return Err(Error::Unknown);
        }

        #[allow(unused_mut)]
        let mut data = std::fs::read(path)?;
        #[cfg(feature = "unzip")]
        {
            use std::io::Read;
            if filename.unwrap().to_str().unwrap().ends_with(".gz") {
                // gunzip first
                let mut gzd = flate2::read::GzDecoder::new(&data[..]);
                let mut decoded_data = Vec::new();
                gzd.read_to_end(&mut decoded_data)?;
                data = decoded_data;
            }
        }

        Font::parse_font_data(&data)
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns number of available characters in the font
    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn get_char(&self, c: char) -> Option<Vec2d<u8>> {
        let cn = c as usize;
        if cn > self.data.len() {
            return None;
        }

        let mut d = Vec::with_capacity(self.data[cn].len() * (self.byte_width * 8));

        let row = &self.data[cn];
        for h in 0..self.height {
            for bit in 0..self.width {
                let bbb = row[h * self.byte_width + bit / 8];
                d.push((bbb >> (7 - (bit % 8))) & 0b1);
            }
        }

        Some(Vec2d {
            d,
            height: self.height,
            width: self.width,
        })
    }

    pub fn print_char(&self, c: char) {
        let c = self.get_char(c).unwrap();
        println!("{:-<1$}", "", c.width + 2);
        for h in 0..c.height {
            print!("|");
            for w in 0..c.width {
                let what = if c.d[h * c.width + w] != 0 { "X" } else { " " };
                print!("{}", what);
            }
            println!("|");
        }
        println!("{:-<1$}", "", c.width + 2);
    }

    fn parse_font_data(raw_data: &[u8]) -> Result<Font, Error> {
        if raw_data.is_empty() {
            return Err(Error::InvalidFontFormat);
        }

        let height;
        let width;
        let byte_width;
        let number: u32;
        let mut data = raw_data.iter();
        let mode = match *data.next().unwrap() {
            0x36 => 1,
            0x72 => 2,
            _ => return Err(Error::InvalidFontFormat),
        };
        if mode == 1 {
            if raw_data.len() < 4 {
                return Err(Error::InvalidFontFormat);
            }
            if *data.next().unwrap() != 0x04 {
                return Err(Error::InvalidFontFormat);
            }
            number = match *data.next().unwrap() {
                0 => 256,
                1 => 512,
                2 => 256,
                3 => 512,
                _ => return Err(Error::InvalidFontFormat),
            };
            height = *data.next().unwrap();
            width = 8;
            byte_width = 1;
        } else {
            if raw_data.len() < 32 {
                return Err(Error::InvalidFontFormat);
            }
            if *data.next().unwrap() != 0xb5
                || *data.next().unwrap() != 0x4a
                || *data.next().unwrap() != 0x86
            {
                return Err(Error::InvalidFontFormat);
            }
            let version = get_data(&mut data, 4);
            if version != [0, 0, 0, 0] {
                return Err(Error::InvalidFontFormat);
            }
            let offset = as_le_u32(&mut data);
            if offset != 0x20 {
                return Err(Error::InvalidFontFormat);
            }
            let _flags = get_data(&mut data, 4);
            number = *data.next().unwrap() as u32 + *data.next().unwrap() as u32 * 256;
            let no_chars = as_le_u16(&mut data);
            if no_chars as u32 > 64 * 1024 {
                return Err(Error::InvalidFontFormat);
            }
            let _sizeof_char = as_le_u32(&mut data);
            height = as_le_u32(&mut data) as u8;
            width = as_le_u32(&mut data) as u8;
            byte_width = (width + 7) / 8;
            assert!(width <= byte_width * 8);
        }

        // println!(
        //     "Parsing psf mode {} font file, with {} characters {} x {} (width x height) [bw={}]",
        //     &mode, &number, &width, &height, &byte_width
        // );

        let mut vvv: Vec<Vec<u8>> = Vec::with_capacity(number as usize);
        for n in 0..number {
            vvv.push(Vec::with_capacity(height as usize * byte_width as usize));
            for _ in 0..height {
                for _ in 0..byte_width {
                    vvv[n as usize].push(*data.next().unwrap());
                }
            }
            assert_eq!(vvv[n as usize].len(), height as usize * byte_width as usize);
        }

        Ok(Font {
            data: vvv,
            width: width as usize,
            height: height as usize,
            byte_width: byte_width as usize,
        })
    }
}

fn as_le_u32(data: &mut std::slice::Iter<u8>) -> u32 {
    (*data.next().unwrap() as u32)
        | (*data.next().unwrap() as u32) << 8
        | (*data.next().unwrap() as u32) << 16
        | (*data.next().unwrap() as u32) << 24
}

fn as_le_u16(data: &mut std::slice::Iter<u8>) -> u16 {
    (*data.next().unwrap() as u16) | (*data.next().unwrap() as u16) << 8
}

fn get_data(data: &mut std::slice::Iter<u8>, count: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(count);
    for _ in 0..count {
        v.push(*data.next().unwrap());
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_path() {
        assert!(Font::new_from_str("blah").is_err());
        assert!(Font::new(std::path::Path::new("foo")).is_err());
    }
}
