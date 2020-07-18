use crate::bits2d::Bits2D;
use std::iter::{IntoIterator, Iterator};
use std::fmt;
use std::io::BufRead;
use std::convert::TryInto;

use png;

type RGBA = (u8,u8,u8,u8);

const COLORS : [RGBA; 13] =
    [
      (  0,  0,  0,200),
      (106,168, 82,200),
      (224, 73, 87,200),
      (125, 97,186,200),
      (255,122, 66,200),
      ( 40,120,181,200),
      (245,223,113,200),
      (194,180,234,200),
      (249,207,221,200),
      (177,223,243,200),
      (198,227,171,200),
      (244,234,150,200),
      (255,199,174,200),
    ];
const COLOR_COORD : RGBA = (255, 255, 255, 100);


#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Point(pub i64, pub i64);

pub struct Screen {
    pixels: Bits2D,
    xstart: i64, // can be 0 or negative
    ystart: i64, // can be 0 or negative
}

pub fn point_from_terminal(xstart : i64, ystart : i64) -> Option<Point> {
    // println!("Coordinate offsets on overlay image were ({}, {})", xstart, ystart);
    println!("Type image coordinates, separated by a space (i.e. as given in an image viewer)");
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
                    (Ok(x), (Ok(y))) => {
                        let p = Point(x,y); // For type infer
                        return Some(Point(x+xstart,y+ystart))
                    }
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
                write!(f, "{}", if self.at_abs(x, y) { '*' } else { ' ' })?
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





pub struct Overlay{
    screens: Vec<Screen>,  // vec of rows
    pub width: u32,
    pub height: u32,
    pub xstart: i64, // can be 0 or negative
    pub ystart: i64, // can be 0 or negative
}

impl Overlay {

    pub fn new(screens : Vec<Screen>) -> Overlay {
        let xstart = (&screens).into_iter().map(|s| s.xstart).min().unwrap();
        let ystart = (&screens).into_iter().map(|s| s.ystart).min().unwrap();
        let xend   = (&screens).into_iter().map(|s| s.width() as i64 + s.xstart).max().unwrap();
        let yend   = (&screens).into_iter().map(|s| s.height() as i64 + s.ystart).max().unwrap();
        let width  = (xend - xstart) as u32;
        let height = (yend - ystart) as u32;
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
                let ptr = 4*(ly*width + lx);
                let mut coli = 0;
                let mut color =
                    if x == 0 || y == 0 {
                        COLOR_COORD
                    } else {
                        RGBA::default()
                    };
                for i in 0..self.screens.len() {
                    if self.at(x, y, i) {
                        color = blend_colors(color, COLORS[i+1]);
                    }
                }
                color = flatten_color(color);
                data[ptr] = color.0;
                data[ptr+1] = color.1;
                data[ptr+2] = color.2;
                data[ptr+3] = color.3;
            }
        }
        w.write_image_data(&data).unwrap();
    }   
}



fn fto8(f : f32) -> u8 {
    let i = (f * 256.) as i64;
    if i < 0 {
        0
    } else if i > 255 {
        255
    } else {
        i as u8
    }
}

fn fof8(c : u8) -> f32 {
    (c as f32) / 256.
}

fn blend_colors(a : RGBA, b : RGBA) -> RGBA {
       // var a = c1.a + c2.a*(1-c1.a);
       // return {
       //   r: (c1.r * c1.a  + c2.r * c2.a * (1 - c1.a)) / a,
       //   g: (c1.g * c1.a  + c2.g * c2.a * (1 - c1.a)) / a,
       //   b: (c1.b * c1.a  + c2.b * c2.a * (1 - c1.a)) / a,
       //   a: a
       // }
    let r_a = fof8(a.3)  + fof8(b.3)*(1.-fof8(a.3));
    let rr = fto8( (fof8(a.0) * fof8(a.3) + fof8(b.0) * fof8(b.3) * (1. - fof8(a.3)) )/r_a );
    let rg = fto8( (fof8(a.1) * fof8(a.3) + fof8(b.1) * fof8(b.3) * (1. - fof8(a.3)) )/r_a );
    let rb = fto8( (fof8(a.2) * fof8(a.3) + fof8(b.2) * fof8(b.3) * (1. - fof8(a.3)) )/r_a );
    let r = (rr, rg, rb, fto8(r_a));
    r
}

fn flatten_color(a : RGBA) -> RGBA {
    let alpha = fof8(a.3);
    (fto8(fof8(a.0)/alpha),
     fto8(fof8(a.1)/alpha),
     fto8(fof8(a.2)/alpha),
     255)
}
