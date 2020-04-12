use tcod::colors::*;
use tcod::{Map as NavMap};
use tcod::pathfinding::{AStar};

use generational_arena::{Index as EntityIndex};

use crate::entity::{Space};
use crate::position::*;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Dimension {
    pub width:  u32,
    pub height: u32
}

impl Dimension {
    pub fn new(width: u32, height: u32) -> Self {
        Dimension {
            width,
            height
        }
    }

    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Traverse {
    Ground,
    Water,
    Wall
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TileKind {
    Floor,
    Wall,
    Ocean
}

#[derive(Debug, Clone)]
pub struct Tile {
    traverse:   Traverse,
    fore_color: Color,
    back_color: Color,
    glyph:      char
}

impl Tile {
    pub fn new(kind: TileKind) -> Self {
        match kind {
            TileKind::Floor => Tile {
                traverse:   Traverse::Ground,
                fore_color: DARK_GREY,
                back_color: BLACK,
                glyph:      '.'
            },

            TileKind::Wall => Tile {
                traverse:   Traverse::Wall,
                fore_color: DARK_GREY,
                back_color: DARK_GREY,
                glyph:      ' '
            },

            TileKind::Ocean => Tile {
                traverse:   Traverse::Water,
                fore_color: DARKER_BLUE,
                back_color: DARKEST_BLUE,
                glyph:      '~'
            },
        }
    }

    pub fn traverse(&self) -> Traverse {
        self.traverse
    }

    pub fn fore_color(&self) -> Color {
        self.fore_color
    }

    pub fn back_color(&self) -> Color {
        self.back_color
    }

    pub fn glyph(&self) -> char {
        self.glyph
    }

    pub fn is_ground(&self) -> bool {
        self.traverse == Traverse::Ground
    }

    pub fn is_water(&self) -> bool {
        self.traverse == Traverse::Water
    }

    pub fn is_wall(&self) -> bool {
        self.traverse == Traverse::Wall
    }
}

#[derive(Debug)]
pub struct Board {
    size:         Dimension,
    tiles:        Vec<Tile>,
    pub entities: Vec<Option<EntityIndex>>
}

impl Board {
    pub fn new(size: Dimension) -> Self {
        Board {
            size: size,
            tiles:    {
                let mut tiles = vec![Tile::new(TileKind::Floor); size.area() as usize];
                for x in 0..size.width {
                    tiles[x as usize] = Tile::new(TileKind::Wall);
                    tiles[(x + size.width * (size.height - 1)) as usize] = Tile::new(TileKind::Wall);
                }
        
                for y in 0..size.height {
                    tiles[(size.width * y) as usize] = Tile::new(TileKind::Wall);
                    tiles[(size.width - 1 + size.width * y) as usize] = Tile::new(TileKind::Wall);
                }
                
                tiles
            },

            entities: vec![None; size.area() as usize]
        }
    }

    pub fn size(&self) -> Dimension {
        self.size
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn to_index(&self, position: Position) -> Option<usize> {
        let index = position.x + position.y * self.size.width as i32;
        if index >= 0 && index < self.tiles.len() as i32 {
            Some(index as usize)
        } else {
            None
        }
    }

    pub fn to_index_unchecked(&self, position: Position) -> usize {
        (position.x + position.y * self.size.width as i32) as usize
    }

    pub fn in_bounds(&self, position: Position) -> bool {
        position.x >= 0 && position.x < self.size.width  as i32 &&
        position.y >= 0 && position.y < self.size.height as i32
    }

    pub fn tile_at(&self, position: Position) -> Option<&Tile> {
        if let Some(index) = self.to_index(position) {
            Some(&self.tiles[index])
        } else {
            None
        }
    }

    pub fn entity_at(&self, position: Position) -> Option<EntityIndex> {
        if let Some(index) = self.to_index(position) {
            self.entities[index]
        } else {
            None
        }
    }

    pub fn navigation_map(&self, space: Option<Space>) -> NavMap {
        let mut map = NavMap::new(self.width() as i32, self.height() as i32);
        for y in 0..self.height() {
            for x in 0..self.width() {
                let tile = &self.tiles[(x + y * self.width()) as usize];

                let can_traverse = {
                    if let Some(space) = space {
                        space.can_traverse(tile.traverse())
                    } else {
                        !tile.is_wall()
                    }
                };

                map.set(x as i32, y as i32, true, can_traverse);
            }
        }

        map
    }

    pub fn in_range(&self, origin: Position, target: Position, range: u32, space: Option<Space>) -> bool {
        let mut astar = {
            let map = self.navigation_map(space);
            AStar::new_from_map(map, 0.0)
        };

        if astar.find(origin.into(), target.into()) {
            astar.walk().count() as u32 <= range
        } else {
            false
        }
    }
}