use crate::bits2d::Bits2D;
use std::iter::{IntoIterator, Iterator};
use std::fmt;
use std::io::BufRead;
use std::convert::TryInto;

use png;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Point(pub i64, pub i64);

pub struct Screen {
    pixels: Bits2D,
    xstart: i64, // can be 0 or negative
    ystart: i64, // can be 0 or negative
}

pub fn point_from_terminal() -> Option<Point> {
    println!("Type coordinates, separated by a space");
    let mut buf = String::new();
    let mut stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    loop {
        if stdin.read_line(&mut buf).unwrap() == 0 {
            return None
        }
        let words: Vec<_> = buf.split_whitespace().collect();
        match &words[..] {
            [x, y] => {
                match (x.parse(), y.parse()) {
                    (Ok(x), (Ok(y))) => return Some(Point(x, y)),
                    _ => ()
                }
            }
            _ => ()
        }
        println!("Bad input. Try again.")
    }
}

pub fn screen_from_list(points: Vec<Point>) -> Screen {
    let mut min_x: i64 = 0;
    let mut min_y: i64 = 0;
    let mut max_x: i64 = 0;
    let mut max_y: i64 = 0;
    for &Point(x, y) in &points {
        max_x = max_x.max(x);
        max_y = max_y.max(y);
        min_x = min_x.min(x);
        min_y = min_y.min(y);
    }

    let mut pixels = Bits2D::new(
        (max_x - min_x + 1).try_into().unwrap(),
        (max_y - min_x + 1).try_into().unwrap()
    );
    for &Point(x, y) in &points {
        pixels.set((x - min_x).try_into().unwrap(), (y - min_y).try_into().unwrap());
    }
    Screen { pixels, xstart: min_x, ystart: min_y }
}


impl Screen {

    fn width(&self) -> u32 {
        self.pixels.length1()
    }

    fn height(&self) -> u32 {
        self.pixels.length2()
    }

    fn at(&self, x : u32, y : u32) -> bool {
        self.pixels[(x,y)]
    }

    fn as_linear_vector(&self) -> Vec<u8> {
        let width = self.width() as usize;
        let height = self.height() as usize;
        let mut data = vec![0u8; width * height];
        for (i, cell) in data.iter_mut().enumerate() {
            let x = i % width;
            let y = i / width / 4;
            *cell = (if self.at(x as u32, y as u32) { 255 } else { 0 });
        }
       data 
    }

    fn dump_image(&self, file_name: &str) {
        let w = std::fs::File::create(file_name).unwrap();
        let w = std::io::BufWriter::new(w);
        let mut encoder = png::Encoder::new(w, self.width(), self.height());
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        let mut w = encoder.write_header().unwrap();
        w.write_image_data(&self.as_linear_vector()).unwrap();
    }
}


impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let x_frame = |x: u32| if x as i64 == -self.xstart { '|' } else { '-' };
        write!(f, " ")?;
        for x in 0..self.width() {
            write!(f, "{}", x_frame(x))?
        }
        writeln!(f, " ")?;

        for y in 0..self.height() {
            let y_frame = if y as i64 == -self.ystart { '-' } else { '|' };
            write!(f, "{}", y_frame)?;
            for x in 0..self.width() {
                write!(f, "{}", if self.at(x, y) { '*' } else { ' ' })?
            }
            writeln!(f, "{}", y_frame)?;
        }

        write!(f, " ")?;
        for x in 0..self.width() {
            write!(f, "{}", x_frame(x))?
        }
        writeln!(f, " ")?;

        Ok(())
    }
}
