use vector2d::Vector2D;

pub type XY = Vector2D<i64>;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct SV {
    pub s: XY,
    pub v: XY,
}

pub fn acceleration_at(pos: XY) -> XY {
    if pos.x.abs() == pos.y.abs() {
        XY { x: - pos.x.signum(), y: - pos.y.signum() }
    } else if (pos.x.abs() > pos.y.abs()) {
        XY { x: - pos.x.signum(), y: 0 }
    } else {
        XY { x: 0, y: - pos.y.signum() }
    }
}

impl SV {
    /// Returns None if we crash into the planet
    pub fn drift(&mut self) {
        self.v += acceleration_at(self.s);
        self.s += self.v;
    }

    pub fn thrust(&mut self, direction_unit_vector: XY) {
        self.v -= direction_unit_vector;
    }

    /// If we crash into the planet, the returned iterator includes the position
    /// where that happens before yielding no more values. If we don't crash, the
    /// returned iterator stops when it's been at least a whole revolution around
    /// the planet.
    pub fn one_orbit_positions(&self, planet_radius: i64, max_steps: i64) -> OneOrbitPositions {
        OneOrbitPositions {
            sv: self.clone(),
            planet_radius,
            steps_left: max_steps,
            last_quadrant: quadrant_of(self.s),
            quadrants_left: 5, // we'll visit one quadrant twice
        }
    }
}

pub fn collided_with_planet(planet_radius: i64, pos: XY) -> bool {
    let absx = pos.x.abs();
    let absy = pos.y.abs();
    absx <= planet_radius && absy <= planet_radius
}

// /// Gives the distance to the planet as measured by the max-norm (infinity norm)
// pub fn dist_to_planet(planet_radius: i64, pos: XY) -> i64 {
//     let absx = pos.x.abs();
//     let absy = pos.y.abs();
//     let pos = (); // shadow
//     if absx <= planet_radius && absy <= planet_radius {
//         // Collision
//         0
//     } else {
//         (absx - planet_radius).max(absy - planet_radius)
//     }
// }

/// Returns {1, 2, 3, 4}, where (+x, 0) is in quadrant 1, (0, +y) is in quadrant
/// 2, etc., and (0,0) is defined to be in quadrant 1.
pub fn quadrant_of(pos: XY) -> u8 {
    if pos.x > 0 && pos.y >= 0 {
        1
    } else if pos.x <= 0 && pos.y > 0 {
        2
    } else if pos.x < 0 && pos.y <= 0 {
        3
    } else if pos.x >= 0 && pos.y < 0 {
        4
    } else { // origin; defined arbitrarily to be 1
        1
    }
}

pub const NONZERO_THRUSTS : [XY; 8] = [
    XY { x: 1, y: 0 },
    XY { x: 1, y: 1 },
    XY { x: 0, y: 1 },
    XY { x:-1, y: 1 },
    XY { x: -1, y: 0 },
    XY { x: -1, y: -1 },
    XY { x: 0, y: -1 },
    XY { x: 1, y: -1 },
];

pub struct OneOrbitPositions{
    sv: SV,
    planet_radius: i64,
    steps_left: i64,
    last_quadrant: u8,
    quadrants_left: u8,
}

impl Iterator for OneOrbitPositions {
    type Item = XY;

    fn next(&mut self) -> Option<Self::Item> {
        if collided_with_planet(self.planet_radius, self.sv.s) {
            None
        } else {
            self.sv.drift();
            let quadrant = quadrant_of(self.sv.s);
            if self.last_quadrant != quadrant {
                self.last_quadrant = quadrant;
                self.quadrants_left -= 1;
            }
            self.steps_left -= 1;

            if self.steps_left <= 0 || self.quadrants_left <= 0 {
                None
            } else {
                Some(self.sv.s)
            }
        }
    }
}