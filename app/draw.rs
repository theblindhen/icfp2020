use crate::bits2d::Bits2D;
use std::iter::{IntoIterator, Iterator};
use std::fmt;
use std::io::BufRead;
use std::convert::TryInto;

use png;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Point(pub i64, pub i64);

pub struct Screen(Bits2D);

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

pub fn image_from_list(points: Vec<Point>) -> Screen {
    let mut max_x: i64 = 0;
    let mut max_y: i64 = 0;
    for &Point(x, y) in &points {
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    let mut image = Bits2D::new((max_x + 1).try_into().unwrap(), (max_y + 1).try_into().unwrap());
    for &Point(x, y) in &points {
        image.set(x.try_into().unwrap(), y.try_into().unwrap());
    }
    Screen(image)
}


impl Screen {

    fn width(&self) -> u32 {
        self.0.length1()
    }

    fn height(&self) -> u32 {
        self.0.length2()
    }

    fn at(&self, x : u32, y : u32) -> bool {
        self.0[(x,y)]
    }

    pub fn dump_image(&self, file_name: &str, rgba: (u8,u8,u8,u8)) {
        let w = std::fs::File::create(file_name).unwrap();
        let w = std::io::BufWriter::new(w);
        let mut encoder = png::Encoder::new(w, self.width(), self.height());
        encoder.set_color(png::ColorType::RGBA);
        encoder.set_depth(png::BitDepth::Eight);
        let mut w = encoder.write_header().unwrap();

        let width = self.width() as usize;
        let height = self.height() as usize;
        let mut data = vec![0u8; 4*width * height];
        let (r,g,b,a) = rgba;
        for x in 0..width {
            for y in 0..height {
                let ptr = 4*(y*width + x);
                if self.at(x as u32, y as u32) {
                    data[ptr] = r;
                    data[ptr+1] = g;
                    data[ptr+2] = b;
                    data[ptr+3] = a;
                } else {
                    data[ptr] = 0;
                    data[ptr+1] = 0;
                    data[ptr+2] = 0;
                    data[ptr+3] = 255;
                }
            }
        }
        w.write_image_data(&data).unwrap();
    }
}


impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " ")?;
        for x in 0..self.width() {
            write!(f, "-")?
        }
        writeln!(f, " ")?;

        for y in 0..self.height() {
            write!(f, "|")?;
            for x in 0..self.width() {
                write!(f, "{}", if self.at(x, y) { '*' } else { ' ' })?
            }
            writeln!(f, "|")?
        }

        write!(f, " ")?;
        for x in 0..self.width() {
            write!(f, "-")?
        }
        writeln!(f, " ")?;

        Ok(())
    }
}
