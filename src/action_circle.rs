use tcod::pathfinding::{AStar};
use crate::{Board, Position, Space};
use std::collections::{HashMap};

pub struct ActionCircle {
    positions: HashMap<Position, u32>
}

/// An ActionCircle is a collection of positions that represent an area
/// that a unit or building can interact with.
impl ActionCircle {
    pub fn new(origin: Position, range: u32, space: Option<Space>, board: &Board) -> Self {
        let positions = {
            let mut positions = HashMap::new();

            let mut astar = {
                let map = board.navigation_map(space);
                AStar::new_from_map(map, 0.0)
            };

            let radius = origin.radius(range as i32);
            for position in radius {
                if board.in_bounds(position) &&
                   astar.find(origin.into(), position.into()) {

                    positions.insert(
                        position,
                        astar.walk().count() as u32
                    );
                }
            }

            positions
        };
        
        ActionCircle {
            positions
        }
    }

    /// Whether the circle contains the position.
    pub fn contains(&self, position: Position) -> bool {
        self.positions.contains_key(&position)
    }

    /// The number of actions required to reach the position.
    pub fn cost_to(&self, position: Position) -> Option<u32> {
        match self.positions.get(&position) {
            Some(cost) => Some(*cost),
            None       => None
        }
    }
}

impl IntoIterator for ActionCircle {
    type Item = (Position, u32);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut result = Vec::new();
        
        for (position, cost) in self.positions {
            result.push((position, cost));
        }

        result.into_iter()
    }
}