//! Reads psf (console) fonts. Exposes very simple interface for displaying
//! the glyphs.
//!
//! Exposing of the glyph data is simple and easy to use:
//! ```
//! use psf::Font;
//!
//! let the_font = Font::new("<path>");
//! if let Ok(font) = the_font {
//!     let c = font.get_char('X');
//!     if let Some(c) = c {
//!         println!("{:-<1$}", "", c.width() + 2);
//!         for h in 0..c.height() {
//!            print!("|");
//!            for w in 0..c.width() {
//!                let what = if c.get(w, h).unwrap() { "X" } else { " " };
//!                print!("{}", what);
//!            }
//!            println!("|");
//!        }
//!        println!("{:-<1$}", "", c.width() + 2);
//!     }
//! }
//! ```

/// Stores information about specific loaded font, including number of
/// available characters, and each character width and height.
pub struct Font {
    raw_data: Vec<u8>,
    font_data_offset: usize,
    count: usize,
    width: usize,
    height: usize,
    byte_width: usize,
}

/// Store information about specific glyph.
// #[derive(Debug)]
pub struct Glyph<'a> {
    d: GlyphData<'a>,
    h: usize,
    w: usize,
    bw: usize,
}

enum GlyphData<'a> {
    ByCopy(Vec<u8>),
    ByRef(&'a [u8]),
}

impl<'a> Glyph<'a> {
    /// Returns if specific point is set (`true`) or not (`false`).
    ///
    /// `x` specifies the point from `0..self.width`
    ///
    /// `y` specifies the point from `0..self.height`
    pub fn get(&self, x: usize, y: usize) -> Option<bool> {
        if x > self.w || y > self.h {
            None
        } else {
            let bit = match &self.d {
                GlyphData::ByCopy(d) => d[y * self.bw + x / 8],
                GlyphData::ByRef(d) => d[y * self.bw + x / 8],
            };
            // let bit = self.d[y * self.bw + x / 8];
            Some((bit >> (7 - (x % 8)) & 0b1) != 0)
        }
    }

    /// Returns width of the glyph
    pub fn width(&self) -> usize {
        self.w
    }

    /// Returns height of the glyph
    pub fn height(&self) -> usize {
        self.h
    }
}

/// Simple error type.
#[derive(Debug, Copy, Clone)]
pub enum Error {
    /// Unspecified error for now
    Unknown,
    /// File doesn't exists
    FileNotFound,
    /// Failure to open and/or read the file itself
    FileIo,
    /// Invalid or unsupported file format
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
    /// Creates a new font for specific path.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Font, Error> {
        Font::_new(path.as_ref())
    }

    fn _new(path: &std::path::Path) -> Result<Font, Error> {
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
            if data[0] == 0x1f && data[1] == 0x8b {
                // gunzip first
                let mut gzd = flate2::read::GzDecoder::new(&data[..]);
                let mut decoded_data = Vec::new();
                gzd.read_to_end(&mut decoded_data)?;
                data = decoded_data;
            }
        }

        Font::parse_font_data(data)
    }

    /// Returns height of every glyph
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns width of every glyph
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns number of available characters in the font
    pub fn size(&self) -> usize {
        self.count
    }

    /// Returns [`Glyph`] data for specific character. If it's not present in the
    /// font, [`None`] is returned.
    pub fn get_char(&self, c: char) -> Option<Glyph> {
        let cn = c as usize;
        if cn > self.count {
            return None;
        }

        let char_byte_length = self.height * self.byte_width;
        let offset = self.font_data_offset + cn * char_byte_length;
        Some(Glyph {
            d: GlyphData::ByRef(&self.raw_data[offset..offset + char_byte_length]),
            h: self.height,
            w: self.width,
            bw: self.byte_width,
        })
    }

    /// Returns [`Glyph`] data for specific character. If it's not present in the
    /// font, [`None`] is returned. Contains copy of the data, so can be used even when
    /// [`Font`] is destroyed.
    pub fn get_char_owned<'b>(&self, c: char) -> Option<Glyph<'b>> {
        let cn = c as usize;
        if cn > self.count {
            return None;
        }

        let char_byte_length = self.height * self.byte_width;
        let offset = self.font_data_offset + cn * char_byte_length;
        Some(Glyph {
            d: GlyphData::ByCopy(self.raw_data[offset..offset + char_byte_length].to_vec()),
            h: self.height,
            w: self.width,
            bw: self.byte_width,
        })
    }
    /// Prints specified character to standard output using [`print!`]
    pub fn print_char(&self, c: char) {
        let c = self.get_char(c).unwrap();
        println!("{:-<1$}", "", c.width() + 2);
        for h in 0..c.height() {
            print!("|");
            for w in 0..c.width() {
                let what = if c.get(w, h).unwrap() { "X" } else { " " };
                print!("{}", what);
            }
            println!("|");
        }
        println!("{:-<1$}", "", c.width() + 2);
    }

    fn parse_font_data(raw_data: Vec<u8>) -> Result<Font, Error> {
        if raw_data.is_empty() {
            return Err(Error::InvalidFontFormat);
        }

        let height;
        let width;
        let byte_width;
        let count: u32;
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
            count = match *data.next().unwrap() {
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
            count = *data.next().unwrap() as u32 + *data.next().unwrap() as u32 * 256;
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

        let font_data_offset = raw_data.len() - data.as_slice().len();
        if mode == 1 {
            assert_eq!(font_data_offset, 4);
        } else {
            assert_eq!(font_data_offset, 32);
        }
        Ok(Font {
            raw_data,
            font_data_offset,
            count: count as usize,
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
        assert!(Font::new("blah").is_err());
        assert!(Font::new(std::path::Path::new("foo")).is_err());
    }

    #[test]
    fn data_convert_u32() {
        let data: Vec<u8> = vec![0x22u8, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99];
        let mut it = data.iter();
        assert_eq!(0x55443322, as_le_u32(&mut it));
        assert_eq!(0x99887766, as_le_u32(&mut it));
    }

    #[test]
    fn data_convert_u16() {
        let data: Vec<u8> = vec![0x22u8, 0x33, 0x44, 0x55];
        let mut it = data.iter();
        assert_eq!(0x3322, as_le_u16(&mut it));
        assert_eq!(0x5544, as_le_u16(&mut it));
    }

    #[test]
    fn test_get_data() {
        let data: Vec<u8> = vec![0x22u8, 0x33, 0x44, 0x55, 0x66, 0x77];
        let mut it = data.iter();
        assert_eq!(vec![0x22u8, 0x33, 0x44], get_data(&mut it, 3));
        assert_eq!(vec![0x55u8], get_data(&mut it, 1));
        assert_eq!(vec![0x66u8, 0x77], get_data(&mut it, 2));
    }
}
