use crate::bits2d::Bits2D;
use std::iter::{IntoIterator, Iterator};
use std::fmt;
use std::io::BufRead;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Point(pub u32, pub u32);

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

pub fn image_from_list<Collection>(points: Collection) -> Screen
where
    for<'a> &'a Collection: IntoIterator<Item = Point>,
{
    let mut max_x: u32 = 0;
    let mut max_y: u32 = 0;
    for Point(x, y) in &points {
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    let mut image = Bits2D::new(max_x + 1, max_y + 1);
    for Point(x, y) in &points {
        image.set(x, y);
    }
    Screen(image)
}

impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = &self.0;
        write!(f, " ")?;
        for x in 0..data.length1() {
            write!(f, "-")?
        }
        writeln!(f, " ")?;

        for y in 0..data.length2() {
            write!(f, "|")?;
            for x in 0..data.length1() {
                write!(f, "{}", if data[(x, y)] { '*' } else { ' ' })?
            }
            writeln!(f, "|")?
        }

        write!(f, " ")?;
        for x in 0..data.length1() {
            write!(f, "-")?
        }
        writeln!(f, " ")?;

        Ok(())
    }
}
