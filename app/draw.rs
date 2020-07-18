use crate::bits2d::Bits2D;
use std::iter::{IntoIterator, Iterator};
use std::fmt;
use std::io::BufRead;
use std::convert::TryInto;

use png;

type RGBA = (u8,u8,u8,u8);

const COLORS : [RGBA; 13] =
    [
      (  0,  0,  0,100),
      (106,168, 82,100),
      (224, 73, 87,100),
      (125, 97,186,100),
      (255,122, 66,100),
      ( 40,120,181,100),
      (245,223,113,100),
      (194,180,234,100),
      (249,207,221,100),
      (177,223,243,100),
      (198,227,171,100),
      (244,234,150,100),
      (255,199,174,100),
    ];


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
        (max_y - min_y + 1).try_into().unwrap()
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

    fn at_abs(&self, x : u32, y : u32) -> bool {
        self.pixels[(x,y)]
    }

    fn at_global(&self, x : i64, y : i64) -> bool {
        let lx = x - self.xstart;
        let ly = y - self.ystart;
        if lx < 0 || lx >= self.width() as i64 || ly < 0 || ly >= self.height() as i64 {
            false
        } else {
            self.pixels[(lx as u32,ly as u32)]
        }
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
                if self.at_abs(x as u32, y as u32) {
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

pub struct Overlay{
    screens: Vec<Screen>,  // vec of rows
    width: u32,
    height: u32,
    xstart: i64, // can be 0 or negative
    ystart: i64, // can be 0 or negative
}

impl Overlay {

    pub fn new(screens : Vec<Screen>) -> Overlay {
        let width  = (&screens).into_iter().map(|s| s.width()).max().unwrap();
        let height = (&screens).into_iter().map(|s| s.height()).max().unwrap();
        let xstart = (&screens).into_iter().map(|s| s.xstart).min().unwrap();
        let ystart = (&screens).into_iter().map(|s| s.ystart).min().unwrap();
        Overlay {screens, width, height, xstart, ystart}
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn at(&self, x : i64, y : i64, i : usize) -> bool {
        // Returns whether the i'th screen is true on (x,y)
        self.screens[i].at_global(x, y)
    }

    pub fn dump_image(&self, file_name: &str) {
        let w = std::fs::File::create(file_name).unwrap();
        let w = std::io::BufWriter::new(w);
        let mut encoder = png::Encoder::new(w, self.width(), self.height());
        encoder.set_color(png::ColorType::RGBA);
        encoder.set_depth(png::BitDepth::Eight);
        let mut w = encoder.write_header().unwrap();

        let width = self.width() as usize;
        let height = self.height() as usize;
        let mut data = vec![0u8; 4*width * height];
        for ly in 0..height {
            let y = ly as i64 + self.ystart;
            for lx in 0..width {
                let x = lx as i64 + self.xstart;
                // TODO: Blend screens
                // TODO: Add coordinate systems
                let ptr = 4*(ly*width + lx);
                let mut coli = 0;
                for i in 0..self.screens.len() {
                    if self.at(x, y, i) {
                        coli = i+1;
                        break
                    }
                }
                let (r,g,b,a) = COLORS[coli];
                data[ptr] = r;
                data[ptr+1] = g;
                data[ptr+2] = b;
                data[ptr+3] = a;
            }
        }
        w.write_image_data(&data).unwrap();
    }   
}
