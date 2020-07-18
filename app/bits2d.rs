use std::ops::Index;

// It looks like the bit-vec package is better maintained than fixedbitset
use fixedbitset::FixedBitSet;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Bits2D {
    elem: FixedBitSet,
    length1: u32,
    length2: u32,
}

impl Bits2D {
    pub fn new(length1: u32, length2: u32) -> Self {
        Bits2D {
            elem: FixedBitSet::with_capacity((length1 as usize) * (length2 as usize)),
            length1: length1,
            length2: length2,
        }
    }

    pub fn length1(&self) -> u32 {
        self.length1
    }
    pub fn length2(&self) -> u32 {
        self.length2
    }

    // this function can't be replaced with IndexMut because that doesn't allow
    // a "virtual Bool"
    pub fn assign(&mut self, x1: u32, x2: u32, value: bool) {
        self.elem.set(
            (x1 as usize) + (x2 as usize) * (self.length1 as usize),
            value,
        );
    }

    pub fn set(&mut self, x1: u32, x2: u32) {
        self.assign(x1, x2, true);
    }
}

impl Index<(u32, u32)> for Bits2D {
    type Output = bool;
    fn index(&self, (x1, x2): (u32, u32)) -> &bool {
        &self.elem[(x1 as usize) + (x2 as usize) * (self.length1 as usize)]
    }
}