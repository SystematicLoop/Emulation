use std::ops::{Add, Sub, Mul, Div};
use std::fmt::{self, Display};

use crate::{Dimension};

#[derive(Debug, Default, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Position {
            x,
            y
        }
    }

    pub fn manhatten_distance(&self, other: &Self) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as u32
    }

    pub fn into_world_pos(&self, world_size: Dimension, world_offset: Position) -> Self {
        let world_pos = *self - world_offset;
        world_pos.clamp_inside(
            0, 
            0, 
            world_size.width  as i32 - 1,
            world_size.height as i32 - 1
        )
    }

    pub fn clamp_inside(&self, x: i32, y: i32, w: i32, h: i32) -> Self {
        Position {
            x: self.x.max(x).min(x + w),
            y: self.y.max(y).min(y + h)
        }
    }

    pub fn radius(&self, radius: i32) -> Vec<Position> {
        let mut positions = Vec::new();
        for y in -radius..=radius {
            for x in -radius..=radius {
                if x == 0 && y == 0 {
                    continue;
                }

                let new_position = *self + Position::new(x as i32, y as i32);
                if (*self).manhatten_distance(&new_position) <= radius as u32 {
                    positions.push(new_position);
                }
            }
        }

        positions
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Position { 
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Position { 
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

impl Mul<i32> for Position {
    type Output = Self;

    fn mul(self, scalar: i32) -> Self::Output {
        Position { 
            x: self.x * scalar,
            y: self.y * scalar
        }
    }
}

impl Div<i32> for Position {
    type Output = Self;

    fn div(self, scalar: i32) -> Self::Output {
        Position { 
            x: self.x / scalar,
            y: self.y / scalar
        }
    }
}

impl Into<(i32, i32)> for Position {
    fn into(self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl Into<(u32, u32)> for Position {
    fn into(self) -> (u32, u32) {
        (self.x as u32, self.y as u32)
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}