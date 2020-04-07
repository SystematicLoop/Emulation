extern crate tcod;

use std::fmt::{Display};
use std::ops::{Add, Sub};

use tcod::console::*;
use tcod::input::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;
use tcod::pathfinding::AStar;
use tcod::colors::*;

#[derive(Copy, Clone, PartialEq, Eq)]
struct Vec2 {
    x: i32,
    y: i32
}

impl Vec2 {
    fn new(x: i32, y: i32) -> Self {
        Vec2 {
            x, y
        }
    }

    fn into_world(&self, graphics: &Graphics) -> Self {
        *self - graphics.board_offset.into()
    }

    fn square_distance_to(&self, other: &Self) -> i32 {
        let x = (other.x - self.x).abs();
        let y = (other.y - self.y).abs();

        x + y
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Self) -> Self::Output {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Self) -> Self::Output {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y
        }
    }
}

impl From<(i32, i32)> for Vec2 {
    fn from(tuple: (i32, i32)) -> Self {
        Vec2::new(tuple.0, tuple.1)
    }
}

impl From<(u32, u32)> for Vec2 {
    fn from(tuple: (u32, u32)) -> Self {
        Vec2::new(tuple.0 as i32, tuple.1 as i32)
    }
}

impl Into<(i32, i32)> for Vec2 {
    fn into(self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl Into<(u32, u32)> for Vec2 {
    fn into(self) -> (u32, u32) {
        (self.x as u32, self.y as u32)
    }
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Copy, Clone, PartialEq)]
enum UnitKind {
    Engineer,
    Infantry,
    Missile,
    Humvee,
    Flag,
    Tank
}

#[derive(Copy, Clone)]
enum Space {
    Ground,
    Naval,
    Air
}

impl Space {
    fn can_traverse(&self, traverse: Traverse) -> bool {
        match (self, traverse) {
            (Space::Ground, Traverse::Ground) => true,
            (Space::Ground, Traverse::Water)  => false,
            (Space::Naval,  Traverse::Ground) => false,
            (Space::Naval,  Traverse::Water)  => true,
            (Space::Air,    Traverse::Ground) => true,
            (Space::Air,    Traverse::Water)  => true,
            (_, _)                            => false,
        }
    }
}

#[derive(Copy, Clone)]
struct Unit {
    kind:  UnitKind,
    space: Space,
    team:  Team,
    glyph: char,

    id:       usize, // Is this sane?
    position: Vec2,

    health:      i32,
    health_max:  i32,
    damage:      i32,
    speed:       i32,
    missile:     bool
}

impl Unit {
    fn new(id: usize, position: Vec2, kind: UnitKind, team: Team) -> Self {
        use UnitKind::*;

        match kind {
            Engineer => Unit {
                kind:        Engineer,
                space:       Space::Ground,
                team:        team,
                glyph:       '\u{0080}',
                id:          id,
                position:    position,
                health:      1,
                health_max:  1,
                damage:      1,
                speed:       2,
                missile:     false
            },

            Infantry => Unit {
                kind:        Infantry,
                space:       Space::Ground,
                team:        team,
                glyph:       '\u{0081}',
                id:          id,
                position:    position,
                health:      2,
                health_max:  2,
                damage:      1,
                speed:       2,
                missile:     false
            },

            Missile => Unit {
                kind:        Missile,
                space:       Space::Air,
                team:        team,
                glyph:       '\u{0082}',
                id:          id,
                position:    position,
                health:      3,
                health_max:  3,
                damage:      5,
                speed:       3,
                missile:     true
            },

            Humvee => Unit {
                kind:        Humvee,
                space:       Space::Ground,
                team:        team,
                glyph:       '\u{0083}',
                id:          id,
                position:    position,
                health:      3,
                health_max:  3,
                damage:      1,
                speed:       3,
                missile:     false
            },

            Flag => Unit {
                kind:        Flag,
                space:       Space::Ground,
                team:        team,
                glyph:       '\u{0084}',
                id:          id,
                position:    position,
                health:      1,
                health_max:  1,
                damage:      0,
                speed:       0,
                missile:     false
            },

            Tank => Unit {
                kind:        Tank,
                space:       Space::Ground,
                team:        team,
                glyph:       '\u{0085}',
                id:          id,
                position:    position,
                health:      4,
                health_max:  4,
                damage:      2,
                speed:       2,
                missile:     false
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Team {
    Red,
    Blue,
    Green,
    Yellow
}

impl Team {
    fn color(&self) -> Color {
        match self {
            Team::Red    => RED,
            Team::Blue   => BLUE,
            Team::Green  => GREEN,
            Team::Yellow => YELLOW
        }
    }
}

#[derive(Copy, Clone)]
enum Traverse {
    Ground,
    Water,
    Wall
}

enum TileKind {
    Floor,
    Wall,
    Ocean
}

#[derive(Copy, Clone)]
struct Tile {
    traverse:   Traverse,
    fore_color: Color,
    back_color: Color,
    glyph:      char
}

impl Tile {
    fn new(kind: TileKind) -> Tile {
        use TileKind::*;
        
        match kind {
            Floor => Tile {
                traverse:   Traverse::Ground,
                fore_color: DARK_GREY,
                back_color: BLACK,
                glyph:      '.'
            },

            Wall => Tile {
                traverse:   Traverse::Wall,
                fore_color: DARK_GREY,
                back_color: DARK_GREY,
                glyph:      ' '
            },

            Ocean => Tile {
                traverse:   Traverse::Water,
                fore_color: DARKER_BLUE,
                back_color: DARKEST_BLUE,
                glyph:      '~'
            }
        }
    }

    fn fore_color(&self) -> Color {
        self.fore_color
    }

    fn back_color(&self) -> Color {
        self.back_color        
    }

    fn glyph(&self) -> char {
        self.glyph
    }

    fn is_ground(&self) -> bool {
        match self.traverse {
            Traverse::Ground => true,
            _                => false
        }
    }

    fn is_wall(&self) -> bool {
        match self.traverse {
            Traverse::Wall => true,
            _              => false
        }
    }
}

#[derive(Debug)]
enum SpawnError {
    InvalidPosition,
    IncompatibleTerrain,
    SpaceOccupied
}

struct Board {
    width:   u32,
    height:  u32,
    tiles:   Vec<Tile>,
    spaces:  Vec<Option<usize>>,
    units:   Vec<Unit>,
}

impl Board {
    fn new(width: u32, height: u32) -> Self {
        let mut tiles = vec![Tile::new(TileKind::Floor); (width * height) as usize];
        for x in 0..width {
            tiles[x as usize] = Tile::new(TileKind::Wall);
            tiles[(x + width * (height - 1)) as usize] = Tile::new(TileKind::Wall);
        }

        for y in 0..height {
            tiles[(width * y) as usize] = Tile::new(TileKind::Wall);
            tiles[(width - 1 + width * y) as usize] = Tile::new(TileKind::Wall);
        }

        let spaces = vec![None; (width * height) as usize];

        Board {
            width:  width,
            height: height,
            tiles:  tiles,
            spaces: spaces,
            units:  Vec::new()
        }
    }

    fn in_bounds(&self, pos: Vec2) -> bool {
        pos.x >= 0 && pos.x < self.width  as i32 &&
        pos.y >= 0 && pos.y < self.height as i32
    }

    fn to_index(&self, pos: Vec2) -> usize {
        (pos.x + pos.y * self.width as i32) as usize
    }

    fn get_tile(&self, pos: Vec2) -> Option<&Tile> {
        if self.in_bounds(pos) {
            self.tiles.get(self.to_index(pos))
        } else {
            None
        }
    }

    fn unit_at(&self, position: Vec2) -> Option<&Unit> {
        let index = self.to_index(position);
        
        if let Some(id) = self.spaces.get(index).unwrap() {
            self.units.get(*id)
        } else {
            None
        }
    }

    fn unit_at_mut(&mut self, position: Vec2) -> Option<&mut Unit> {
        let index = self.to_index(position);
        
        if let Some(id) = self.spaces.get(index).unwrap() {
            self.units.get_mut(*id)
        } else {
            None
        }
    }

    fn spawn(&mut self, position: Vec2, kind: UnitKind, team: Team) -> Result<usize, SpawnError> {
        let id    = self.units.len();
        let index = self.to_index(position);
        let unit  = Unit::new(id, position, kind, team);

        if let Some(tile) = self.get_tile(position) {
            if !unit.space.can_traverse(tile.traverse) {
                return Err(SpawnError::IncompatibleTerrain);
            }
        } else {
            return Err(SpawnError::InvalidPosition);
        }

        if self.spaces[index].is_some() {
            return Err(SpawnError::SpaceOccupied);
        }

        self.units.push(unit);
        self.spaces[index] = Some(id);

        Ok(id)
    }

    fn navigation_map(&self, space: Space) -> tcod::Map {
        let mut map = tcod::Map::new(self.width as i32, self.height as i32);
        for y in 0..self.height {
            for x in 0..self.width {
                let tile = self.get_tile((x, y).into())
                    .unwrap();

                map.set(x as i32, y as i32, true, space.can_traverse(tile.traverse));
            }
        }

        map
    }
}

#[derive(Debug)]
enum DamageError {
    InvalidPosition
}

fn do_damage(board: &mut Board, damage: Damage) -> Result<bool, DamageError> {
    if !board.in_bounds(damage.at) {
        return Err(DamageError::InvalidPosition);
    }
    
    let target_killed = {
        if let Some(unit) = board.unit_at_mut(damage.at) {
            if unit.team != damage.team {
                unit.health -= damage.amount;
                unit.health  = unit.health.max(0);

                if unit.health == 0 {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    };

    if target_killed {
        let index = board.to_index(damage.at);
        board.spaces[index] = None;
    }

    Ok(target_killed)
}

#[derive(Debug)]
enum WarpError {
    InvalidSelection,
    InvalidPosition,
    IncompatibleTerrain,
    SpaceOccupied
}

fn do_warp(board: &mut Board, warp: Warp) -> Result<(), WarpError> {
    let (space, position) = {
        if let Some(unit) = board.units.get(warp.unit) {
            (unit.space, unit.position)
        } else {
            return Err(WarpError::InvalidSelection);
        }
    };

    if let Some(tile) = board.get_tile(warp.to) {
        if !space.can_traverse(tile.traverse) {
            return Err(WarpError::IncompatibleTerrain);
        }
    } else {
        return Err(WarpError::InvalidPosition);
    }

    if let Some(_) = board.unit_at(warp.to) {
        return Err(WarpError::SpaceOccupied);
    }

    let i = board.to_index(position);
    let j = board.to_index(warp.to);

    board.spaces[i] = None;
    board.spaces[j] = Some(warp.unit);
    board.units[warp.unit].position = warp.to;

    Ok(())
}

#[derive(Debug)]
enum MoveError {
    InvalidSelection,
    InvalidPosition,
    IncompatibleTerrain,
    DestinationNotReachable,
    SpaceOccupied
}

fn do_move(board: &mut Board, m: Move) -> Result<(), MoveError> {
    let (space, damage, position, team) = {
        if let Some(unit) = board.units.get(m.unit) {
            (unit.space, unit.damage, unit.position, unit.team)
        } else {
            return Err(MoveError::InvalidSelection);
        }
    };

    let mut astar = {
        let map = board.navigation_map(space);
        AStar::new_from_map(map, 0.0)
    };

    if !astar.find(position.into(), m.to.into()) {
        return Err(MoveError::DestinationNotReachable);
    }

    let mut destination        = m.to;
    let mut before_destination = position;

    // Procedurally walk towards the target until
    // something happens to be in the way.
    for pos in astar.walk() {
        if let Some(_) = board.unit_at(pos.into()) {
            destination = pos.into();
            break;
        } else {
            before_destination = pos.into();
        }
    }
    
    let (had_target, target_killed) = {
        match do_damage(board, Damage { team: team, amount: damage, at: destination }) {
            Ok(killed) => (true, killed),
            Err(_)     => (false, false)
        }
    };

    // println!("[Move] Had Target ({}), Killed ({})", had_target, target_killed);

    let warp_result = {
        if !had_target || target_killed {
            do_warp(board, Warp { unit: m.unit, to: destination })
        } else {
            do_warp(board, Warp { unit: m.unit, to: before_destination })
        }
    };

    match warp_result {
        Err(error) => {
            match error {
                WarpError::InvalidSelection    => Err(MoveError::InvalidSelection),
                WarpError::InvalidPosition     => Err(MoveError::InvalidPosition),
                WarpError::IncompatibleTerrain => Err(MoveError::IncompatibleTerrain),
                WarpError::SpaceOccupied       => Err(MoveError::SpaceOccupied)
            }
        },

        Ok(_) => Ok(())
    }
}

/*
fn do_move(game: &mut Game, mut m: Move) {
    let mut astar = {
        let map = game.board.navigation_map(m.space);
        AStar::new_from_map(map, 0.0)
    };

    if !astar.find(m.origin.into(), m.dest.into()) {
        println!("[Move] Target unreachable.");
        return;
    }

    let mut pos_before_dest = m.origin;

    // Procedurally walk towards the target until
    // something happens to be in the way.
    for pos in astar.walk() {
        if let Some(target) = game.board.get_unit_mut(pos.into()) {
            m.dest = pos.into();
            break;
        } else {
            pos_before_dest = pos.into();
        }
    }

    // An essay can be written on the issues with this function.
    let i = game.board.to_index(m.origin);
    let j = game.board.to_index(m.dest);

    let mut had_target    = false;
    let mut target_killed = false;
    
    if let Pair::Both(tile, other_tile) = index_twice(&mut game.board.tiles, i, j) {
        if other_tile.is_wall() {
            println!("[Move] Blocked by wall.");
            return;
        }
    
        if tile.team() == other_tile.team() {
            println!("[Move] Blocked by friendly.");
            return;
        }

        match (tile.unit(), other_tile.unit_mut()) {
            (Some(unit), Some(other_unit)) => {
                had_target = true;
                (*other_unit).health -= unit.damage;
                if other_unit.health <= 0 {
                    if unit.missile {
                        other_tile.unit = None;
                    } else {
                        other_tile.unit = tile.unit;
                    }

                    tile.unit = None;
                    target_killed = true;
                }
            },
            
            (Some(_), None) => {
                println!("[Move] The target was a floor.");
                other_tile.unit = tile.unit;
                tile.unit       = None;
            },
    
            _ => {
                println!("[Move] No tile selected.");
            }
        }   
    }

    if had_target && !target_killed {
        let j = game.board.to_index(pos_before_dest);
        if let Pair::Both(tile, other_tile) = index_twice(&mut game.board.tiles, i, j) {
            other_tile.unit = tile.unit;
            tile.unit       = None;
        }
    }
}
*/

struct Warp {
    unit: usize,
    to:   Vec2
}

struct Damage {
    team:   Team,
    amount: i32,
    at:     Vec2
}

struct Move {
    unit: usize,
    to:   Vec2
}

struct Movement {
    abs_origin:    Vec2,
    rel_origin:    Vec2,
    range:         i32,
    space:         Space,
    valid_tiles:   Vec<Vec2>,
    checked_tiles: Vec<Vec2>
}

impl Movement {
    fn new(abs_origin: Vec2, range: i32, space: Space) -> Self {
        Movement {
            abs_origin:    abs_origin,
            rel_origin:    abs_origin,
            range:         range,
            space:         space,
            valid_tiles:   vec![],
            checked_tiles: vec![abs_origin]
        }
    }
}

fn movement_query(board: &Board, movement: &mut Movement) {
    let north = movement.rel_origin + Vec2::new( 0, -1);
    let east  = movement.rel_origin + Vec2::new( 1,  0);
    let south = movement.rel_origin + Vec2::new( 0,  1);
    let west  = movement.rel_origin + Vec2::new(-1,  0);

    if let Some(north_tile) = board.get_tile(north) {
        if !movement.checked_tiles.contains(&north) && 
            movement.abs_origin.square_distance_to(&north) <= movement.range {
            
            movement.checked_tiles.push(north);
            if movement.space.can_traverse(north_tile.traverse) {
                movement.valid_tiles.push(north);
                
                let rel_origin = movement.rel_origin;
                movement.rel_origin = north;
                movement_query(&board, movement);
                movement.rel_origin = rel_origin;
            }
        }
    }

    if let Some(east_tile) = board.get_tile(east) {
        if !movement.checked_tiles.contains(&east) && 
            movement.abs_origin.square_distance_to(&east) <= movement.range {
            
            movement.checked_tiles.push(east);
            if movement.space.can_traverse(east_tile.traverse) {
                movement.valid_tiles.push(east);
                
                let rel_origin = movement.rel_origin;
                movement.rel_origin = east;
                movement_query(&board, movement);
                movement.rel_origin = rel_origin;
            }
        }
    }

    if let Some(south_tile) = board.get_tile(south) {
        if !movement.checked_tiles.contains(&south) && 
            movement.abs_origin.square_distance_to(&south) <= movement.range {
            
            movement.checked_tiles.push(south);
            if movement.space.can_traverse(south_tile.traverse) {
                movement.valid_tiles.push(south);
                
                let rel_origin = movement.rel_origin;
                movement.rel_origin = south;
                movement_query(&board, movement);
                movement.rel_origin = rel_origin;
            }
        }
    }

    if let Some(west_tile) = board.get_tile(west) {
        if !movement.checked_tiles.contains(&west) && 
            movement.abs_origin.square_distance_to(&west) <= movement.range {
            
            movement.checked_tiles.push(west);
            if movement.space.can_traverse(west_tile.traverse) {
                movement.valid_tiles.push(west);
                
                let rel_origin = movement.rel_origin;
                movement.rel_origin = west;
                movement_query(&board, movement);
                movement.rel_origin = rel_origin;
            }
        }
    }

    // Post-processing to ensure tiles are reachable within the specified range.
    let map = board.navigation_map(movement.space);

    let mut astar  = AStar::new_from_map(map, 0.0);
    let abs_origin = movement.abs_origin.into();
    let range      = movement.range;

    movement.valid_tiles.retain(|&pos| {
        if astar.find(abs_origin, (pos.x, pos.y)) {
            astar.walk().count() as i32 <= range
        } else {
            false
        }
    });
}

#[derive(PartialEq, Eq)]
enum PlayerState {
    Selecting,
    Moving(usize)
}

struct Game {
    board:     Board,
    state:     PlayerState,
    selection: Option<Vec2>,
    movement:  Option<Movement>,
    mouse_pos: Vec2,
    world_pos: Vec2
}

struct Graphics {
    root:         Root,
    board_offset: (i32, i32),
    board:        Offscreen
}

fn draw(graphics: &mut Graphics, game: &Game) {
    graphics.root.clear();
    
    let unit: Option<&Unit> = {
        match game.state {
            PlayerState::Moving(id) => {
                game.board.units.get(id)
            },

            _ => None
        }
    };

    for y in 0..game.board.height {
        for x in 0..game.board.width {
            let position = Vec2::new(x as i32, y as i32);
            let tile = game.board.get_tile(position)
                .unwrap();

            let fore_color = {
                if let Some(unit) = game.board.unit_at(position) {
                    unit.team.color()
                } else {
                    tile.fore_color()
                }
            };

            let back_color = tile.back_color();
            let glyph      = {
                if let Some(unit) = game.board.unit_at(position) {
                    unit.glyph
                } else {
                    tile.glyph()
                }
            };

            graphics.board.put_char_ex(
                x as i32,
                y as i32,
                glyph,
                fore_color,
                back_color
            );
        }
    }

    // If there is a unit, we want to
    // invert the colour at that tile.
    if let Some(unit) = unit {
        let selection  = unit.position;
        let back_color = graphics.board.get_char_background(selection.x, selection.y);
        let fore_color = graphics.board.get_char_foreground(selection.x, selection.y);
        
        graphics.board.set_char_foreground(
            selection.x,
            selection.y,
            back_color
        );

        graphics.board.set_char_background(
            selection.x,
            selection.y,
            fore_color,
            BackgroundFlag::Set
        );

        // If there is also movement data, draw it.
        if let Some(movement) = &game.movement {
            let mut mouse_in_valid_region = false;

            for pos in &movement.valid_tiles {
                let color = if game.world_pos == *pos {
                    mouse_in_valid_region = true;
                    DARKER_YELLOW
                } else {
                    DARKEST_YELLOW
                };

                graphics.board.set_char_background(
                    pos.x,
                    pos.y,
                    color,
                    BackgroundFlag::Add
                );
            }

            if !mouse_in_valid_region && 
                game.world_pos != selection &&
               !game.board.get_tile(game.world_pos).unwrap().is_wall() {
                
                graphics.board.set_char_background(
                    game.world_pos.x,
                    game.world_pos.y,
                    DARKEST_RED,
                    BackgroundFlag::Set
                );
            } else {
                let map = game.board.navigation_map(unit.space);
                let mut astar = tcod::pathfinding::AStar::new_from_map(map, 0.0);
                if astar.find(selection.into(), game.world_pos.into()) {
                    for pos in astar.walk() {
                        graphics.board.set_char_background(
                            pos.0,
                            pos.1,
                            DARKEST_MAGENTA,
                            BackgroundFlag::Set
                        );
                    }
                }
            }
        }
    }

    blit(&graphics.board, (0, 0), (0, 0), &mut graphics.root, graphics.board_offset, 1.0, 1.0);
}

fn menu_with_options(graphics: &mut Graphics, prompt: String, options: Vec<String>) -> Option<usize> {
    graphics.root.clear();
    graphics.root.print(0, 0, prompt);

    for (i, option) in options.iter().enumerate() {
        graphics.root.print(1, 1 + i as i32, format!("{} {}", i + 1, option));
    }

    graphics.root.flush();

    let key = graphics.root.wait_for_keypress(true);
    let selection = match key.code {
        KeyCode::Number1 => 0,
        KeyCode::Number2 => 1,
        KeyCode::Number3 => 2,
        KeyCode::Number4 => 3,
        KeyCode::Number5 => 4,
        KeyCode::Number6 => 5,
        KeyCode::Number7 => 6,
        KeyCode::Number8 => 7,
        KeyCode::Number9 => 8,
        KeyCode::Number0 => 9,
        _                => return None
    };

    if selection < options.len() {
        Some(selection)
    } else {
        None
    }
}

fn spawn_menu(game: &mut Game, graphics: &mut Graphics) {
    let selection = menu_with_options(graphics, String::from("Spawn/Unit"), vec![
        String::from("Engineer"),
        String::from("Infantry"),
        String::from("Humvee"),
        String::from("Missile"),
        String::from("Flag"),
        String::from("Tank")
    ]);

    let kind = match selection {
        Some(index) => {
            match index {
                0 => UnitKind::Engineer,
                1 => UnitKind::Infantry,
                2 => UnitKind::Humvee,
                3 => UnitKind::Missile,
                4 => UnitKind::Flag,
                5 => UnitKind::Tank,

                _ => return
            }
        },

        None => return
    };

    let selection = menu_with_options(graphics, String::from("Spawn/Unit/Team"), vec![
        String::from("Red"),
        String::from("Blue"),
        String::from("Green"),
        String::from("Yellow")
    ]);

    let team = match selection {
        Some(index) => {
            match index {
                0 => Team::Red,
                1 => Team::Blue,
                2 => Team::Green,
                3 => Team::Yellow,
             
                _ => return
            }
        },

        None => return
    };

    match game.board.spawn(game.world_pos, kind, team) {
        Ok(id)     => println!("[Spawn] Success (ID {}).", id),
        Err(error) => println!("[Spawn] Failure ({:?})", error)
    }
}

fn main() {
    let root = Root::initializer()
        .size(27, 21)
        .title("A Starless Void")
        .font("res/Font 32x32 Extended.png", FontLayout::AsciiInRow)
        .init();

    let mut game = Game {
        board:     Board::new(10, 10),
        state:     PlayerState::Selecting,
        selection: None,
        movement:  None,
        mouse_pos: Vec2::new(0, 0),
        world_pos: Vec2::new(0, 0)
    };

    let mut graphics = Graphics {
        root:  root,
        board_offset: (8, 6),
        board: Offscreen::new(game.board.width as i32, game.board.height as i32)
    };

    game.board.spawn((4, 4).into(), UnitKind::Humvee, Team::Red).unwrap();
    
    while !graphics.root.window_closed() {
        graphics.root.set_default_background(BLACK);
        graphics.root.set_default_foreground(WHITE);
        draw(&mut graphics, &game);

        match game.state {
            PlayerState::Selecting => {
                graphics.root.set_alignment(TextAlignment::Center);
                graphics.root.print(12, 19, "=== Select ===");
                graphics.root.set_alignment(TextAlignment::Left);
            
                if let Some(unit) = game.board.unit_at(game.world_pos) {
                    graphics.root.print(1, 1, format!("HP {}/{}", unit.health, unit.health_max));
                }
            },

            PlayerState::Moving(id) => {
                graphics.root.set_alignment(TextAlignment::Center);
                graphics.root.print(12, 19, "=== Move ===");
                graphics.root.print(12, 18, "Cancel (Esc)");
                graphics.root.set_alignment(TextAlignment::Left);
            
                if let Some(unit) = game.board.units.get(id) {
                    graphics.root.print(1, 1, format!("HP {}/{}", unit.health, unit.health_max));
                }

                if let Some(target) = game.board.unit_at(game.world_pos) {
                    graphics.root.set_default_foreground(MAGENTA);
                    graphics.root.print(1, 2, format!("HP {}/{} (Target)", target.health, target.health_max));
                    graphics.root.set_default_foreground(WHITE);
                }
            }
        }
        
        graphics.root.flush();

        let mut mouse = Mouse::default();
        let mut key   = Key::default();

        match check_for_event(MOUSE | KEY_PRESS) {
            Some((_, Event::Mouse(m))) => {
                mouse = m;
                game.mouse_pos = Vec2::new(mouse.cx as i32, mouse.cy as i32);
                game.world_pos = game.mouse_pos.into_world(&graphics);
                game.world_pos.x = game.world_pos.x.max(0).min(game.board.width  as i32 - 1);
                game.world_pos.y = game.world_pos.y.max(0).min(game.board.height as i32 - 1);
            },

            Some((_, Event::Key(k))) => key = k,
            _ => {  },
        }

        match key {
            Key {
                code: Escape,
                ..
            } => {
                game.selection = None;
                game.movement  = None;
                game.state     = PlayerState::Selecting;
            },

            Key {
                code: Delete,
                ..
            } => {
                /*
                if let Some(tile) = game.board.get_tile_mut(game.world_pos) {
                    tile.unit = None;
                }
                */
            }

            _ => {  }
        }

        if mouse.lbutton_pressed {
            println!("Mouse clicked {}", game.mouse_pos);

            let world_pos = game.mouse_pos.into_world(&graphics);

            // This particular section seems messy/fragile.
            match game.state {
                PlayerState::Selecting => {
                    if let Some(unit) = game.board.unit_at(world_pos) {
                        println!("[Select] Selected unit.");

                        let mut movement = Movement::new(world_pos, unit.speed, unit.space);
                        movement_query(&game.board, &mut movement);

                        game.movement  = Some(movement);
                        game.state     = PlayerState::Moving(unit.id);
                    }
                },

                PlayerState::Moving(id) => {
                    println!("[Move] Attemping to move.");

                    if let Some(movement) = &game.movement {
                        let space = movement.space;
                        if movement.valid_tiles.contains(&world_pos) {
                            match do_move(&mut game.board, Move {
                                unit: id,
                                to:   world_pos
                            }) {
                                Err(error) => println!("[Move] Failure ({:?})", error),
                                Ok(_)      => println!("[Move] Success")
                            }

                            game.selection = None;
                            game.movement  = None;
                            game.state     = PlayerState::Selecting;
                        } else {
                            /*
                            println!("{} was not valid a tile. Valid tiles were:", game.mouse_pos);
                            for pos in &movement.valid_tiles {
                                println!(" {}", pos);
                            }
                            println!("-------------------");
                            */
                        }
                    }
                }
            }
        }
        
        if mouse.rbutton_pressed {
            spawn_menu(&mut game, &mut graphics);
        }
    }
}