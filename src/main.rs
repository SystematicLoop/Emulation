extern crate tcod;

use std::fmt::{Display};
use std::ops::{Add, Sub};

use tcod::console::*;
use tcod::input::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;
use tcod::colors::*;



// Pair<T> and index_twice<T> taken from: 
// https://stackoverflow.com/questions/30073684/how-to-get-mutable-references-to-two-array-elements-at-the-same-time
enum Pair<T> {
    Both(T, T),
    One(T),
    None,
}

fn index_twice<T>(slc: &mut Vec<T>, a: usize, b: usize) -> Pair<&mut T> {
    if a == b {
        slc.get_mut(a).map_or(Pair::None, Pair::One)
    } else {
        if a >= slc.len() || b >= slc.len() {
            Pair::None
        } else {
            // Safe because a and b are in bounds and distinct.
            unsafe {
                let ar = &mut *(slc.get_unchecked_mut(a) as *mut _);
                let br = &mut *(slc.get_unchecked_mut(b) as *mut _);
                Pair::Both(ar, br)
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct Vec2 {
    x: i32,
    y: i32
}

impl Vec2 {
    fn new(x: i32, y: i32) -> Self {
        Vec2 {
            x: x,
            y: y
        }
    }

    fn into_world(&self, graphics: &Graphics) -> Self {
        *self - graphics.board_offset.into()
    }

    fn diagonal_distance_to(&self, other: &Self) -> u32 {
        let x = (other.x - self.x);
        let y = (other.y - self.y);

        ((x * x + y * y) as f32).sqrt().round() as u32
    }

    fn square_distance_to(&self, other: &Self) -> u32 {
        let x = (other.x - self.x).abs();
        let y = (other.y - self.y).abs();

        (x + y) as u32
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
    Humvee
}

// TODO:
//   Refactor units to have:
//     Health, Damage, Speed, Bonus table, and Missile flag.

struct Bonus {
    against: UnitKind,
    damage:  u32
}

#[derive(Copy, Clone)]
struct Unit {
    kind:  UnitKind,
    team:  Team,
    glyph: char,

    health:     i32,
    health_max: i32,
    damage:     i32,
    speed:      u32,
    missile:    bool
}

impl Unit {
    fn new(kind: UnitKind, team: Team) -> Self {
        use UnitKind::*;

        match kind {
            Engineer => Unit {
                kind:       Engineer,
                team:       team,
                glyph:      '\u{0080}',
                health:     1,
                health_max: 1,
                damage:     1,
                speed:      2,
                missile:    false
            },

            Infantry => Unit {
                kind:       Infantry,
                team:       team,
                glyph:      '\u{0081}',
                health:     2,
                health_max: 2,
                damage:     1,
                speed:      2,
                missile:    false
            },

            Missile => Unit {
                kind:       Missile,
                team:       team,
                glyph:      '\u{0082}',
                health:     3,
                health_max: 3,
                damage:     5,
                speed:      3,
                missile:    true
            },

            Humvee => Unit {
                kind:       Humvee,
                team:       team,
                glyph:      '\u{0083}',
                health:     3,
                health_max: 3,
                damage:     2,
                speed:      3,
                missile:    false
            }
        }
    }

    /*
    fn try_attack(&self, other: &Self) -> Attack {
        use UnitKind::*;
        
        if self.rank < other.rank {
            match (self.kind, other.kind) {
                (Engineer, Missile) => Attack::Win,
                _                   => Attack::Loss
            }
        } else if self.rank > other.rank {
            match (self.kind, other.kind) {
                (Missile, Engineer) => Attack::Loss,
                _                   => Attack::Win
            }
        } else {
            Attack::Tie
        }
    }
    */
}

#[derive(Copy, Clone)]
enum Attack {
    Loss,
    Tie,
    Win
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
enum Tile {
    None,
    Wall,
    Some(Unit)
}

impl Tile {
    fn color(&self) -> tcod::colors::Color {
        match self {
            Tile::None       => tcod::colors::DARK_GREY,
            Tile::Wall       => tcod::colors::DARK_GREY,
            Tile::Some(unit) => unit.team.color()
        }
    }

    fn glyph(&self) -> char {
        match self {
            Tile::None       => '.',
            Tile::Wall       => '\u{00DB}',
            Tile::Some(unit) => unit.glyph
        }
    }

    fn unit(&self) -> Option<&Unit> {
        match self {
            Tile::Some(unit) => Some(unit),
            _                => None
        }
    }

    fn unit_mut(&mut self) -> Option<&mut Unit> {
        match self {
            Tile::Some(unit) => Some(unit),
            _                => None
        }
    }

    fn team(&self) -> Option<Team> {
        match self {
            Tile::Some(unit) => Some(unit.team),
            _                => None
        }
    }

    fn wall(&self) -> bool {
        match self {
            Tile::Wall => true,
            _          => false
        }
    }

    fn floor(&self) -> bool {
        match self {
            Tile::None => true,
            _          => false
        }
    }
}

struct Board {
    width:  u32,
    height: u32,
    tiles:  Vec<Tile>
}

impl Board {
    fn new(width: u32, height: u32) -> Self {
        Board {
            width:  width,
            height: height,
            tiles:  vec![Tile::None; (width * height) as usize]
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

    fn get_tile_mut(&mut self, pos: Vec2) -> Option<&mut Tile> {
        if self.in_bounds(pos) {
            let index = self.to_index(pos); // Compiler complains if this is inline with get_mut
            self.tiles.get_mut(index)
        } else {
            None
        }
    }

    fn set_tile(&mut self, pos: Vec2, tile: Tile) {
        if self.in_bounds(pos) {
            let index = self.to_index(pos);
            self.tiles[index] = tile;
        }
    }

    fn get_unit(&self, pos: Vec2) -> Option<&Unit> {
        if let Some(tile) = self.get_tile(pos) {
            tile.unit()
        } else {
            None
        }
    }

    fn spawn(&mut self, pos: Vec2, kind: UnitKind, team: Team) {
        if let Some(tile) = self.get_tile_mut(pos) {
            if tile.floor() {
                *tile = Tile::Some(Unit::new(kind, team));
            }
        }
    }

    fn navigation_map(&self) -> tcod::Map {
        let mut map = tcod::Map::new(self.width as i32, self.height as i32);
        for y in 0..self.height {
            for x in 0..self.width {
                let tile = self.get_tile((x, y).into())
                    .unwrap();

                map.set(x as i32, y as i32, true, !tile.wall());
            }
        }

        map
    }
}

struct Move {
    origin: Vec2,
    dest:   Vec2
}

impl Move {
    fn new(origin: Vec2, dest: Vec2) -> Self {
        Move {
            origin,
            dest
        }
    }
}

fn do_move(board: &mut Board, m: Move) {
    // An essay can be written on the issues with this function.
    let i = board.to_index(m.origin);
    let j = board.to_index(m.dest);
    
    if let Pair::Both(tile, other_tile) = index_twice(&mut board.tiles, i, j) {
        if other_tile.wall() {
            println!("Movement blocked by wall.");
            return;
        }
    
        if tile.team() == other_tile.team() {
            println!("Movement blocked by friendly.");
            return;
        }

        match (tile.unit(), other_tile.unit_mut()) {
            (Some(unit), Some(other_unit)) => {
                (*other_unit).health -= unit.damage;
                if other_unit.health <= 0 {
                    if unit.missile {
                        *other_tile = Tile::None;
                    } else {
                        *other_tile = *tile;
                    }

                    *tile = Tile::None;
                } else {
                    
                }

                /*
                match unit.try_attack(&other_unit) {
                    Attack::Loss => {
                        *tile = Tile::None;
    
                        if other_unit.kind == UnitKind::Missile {
                            *other_tile = Tile::None;    
                        }
                    },
    
                    Attack::Tie => {
                        *tile       = Tile::None;
                        *other_tile = Tile::None;
                    },
    
                    Attack::Win => {
                        println!("Win");
                        
                        if unit.kind == UnitKind::Missile {
                            println!("Was missile");
                            *other_tile = Tile::None;
                        } else {
                            println!("Was not missile");
                            *other_tile = *tile;
                        }

                        *tile = Tile::None;
                    }
                }
                */
            },
            
            (Some(_), None) => {
                println!("The target was a floor.");
                *other_tile = *tile;
                *tile       = Tile::None;
            },
    
            _ => {
                println!("No tile selected.");
            }
        }   
    }
}

struct Movement {
    abs_origin:    Vec2,
    rel_origin:    Vec2,
    range:         u32,
    valid_tiles:   Vec<Vec2>,
    checked_tiles: Vec<Vec2>
}

impl Movement {
    fn new(abs_origin: Vec2, range: u32) -> Self {
        Movement {
            abs_origin:    abs_origin,
            rel_origin:    abs_origin,
            range:         range,
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
            if !north_tile.wall() {
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
            if !east_tile.wall() {
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
            if !south_tile.wall() {
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
            if !west_tile.wall() {
                movement.valid_tiles.push(west);
                
                let rel_origin = movement.rel_origin;
                movement.rel_origin = west;
                movement_query(&board, movement);
                movement.rel_origin = rel_origin;
            }
        }
    }

    // Post-processing to ensure tiles are reachable within the specified range.
    let mut map = board.navigation_map();

    let mut astar  = tcod::pathfinding::AStar::new_from_map(map, 0.0);
    let abs_origin = movement.abs_origin.into();
    let range      = movement.range;

    movement.valid_tiles.retain(|&pos| {
        if astar.find(abs_origin, (pos.x, pos.y)) {
            astar.walk().count() as u32 <= range
        } else {
            false
        }
    });
}

#[derive(PartialEq, Eq)]
enum PlayerState {
    Selecting,
    Moving
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
        if let Some(selection) = game.selection {
            game.board.get_unit(selection)
        } else {
            None
        }
    };

    for y in 0..game.board.height {
        for x in 0..game.board.width {
            let tile = game.board.get_tile(Vec2::new(x as i32, y as i32))
                .unwrap();

            let fore_color = tile.color();
            let back_color = BLACK;
            let glyph = tile.glyph();

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
    if unit.is_some() {
        let selection  = game.selection.unwrap();
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
                    BackgroundFlag::Set
                );
            }

            /*
            let map = game.board.navigation_map();
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
            */

            if !mouse_in_valid_region && game.world_pos != selection {
                graphics.board.set_char_background(
                    game.world_pos.x,
                    game.world_pos.y,
                    DARKEST_RED,
                    BackgroundFlag::Set
                );
            }
        }
    }

    blit(&graphics.board, (0, 0), (0, 0), &mut graphics.root, graphics.board_offset, 1.0, 1.0);
}

fn main() {
    let mut root = Root::initializer()
        .size(27, 27)
        .title("A Starless Void")
        .font("res/Font 16x16 Extended.png", FontLayout::AsciiInRow)
        .init();

    let mut board = Board::new(10, 10);

    for y in 0..board.height {
        for x in 0..board.width {
            if x == 0 || x == board.width - 1 || y == 0 || y == board.height - 1 {
                board.set_tile((x, y).into(), Tile::Wall);
            }
        }
    }

    board.spawn((1, 1).into(), UnitKind::Engineer, Team::Red);
    board.spawn((2, 1).into(), UnitKind::Infantry, Team::Red);
    board.spawn((3, 1).into(), UnitKind::Infantry, Team::Red);
    board.spawn((4, 1).into(), UnitKind::Infantry, Team::Red);
    board.spawn((5, 1).into(), UnitKind::Missile,  Team::Red);
    board.spawn((8, 8).into(), UnitKind::Engineer, Team::Blue);
    board.spawn((7, 8).into(), UnitKind::Infantry, Team::Blue);
    board.spawn((6, 8).into(), UnitKind::Infantry, Team::Blue);
    board.spawn((5, 8).into(), UnitKind::Infantry, Team::Blue);
    board.spawn((4, 8).into(), UnitKind::Missile,  Team::Blue);

    board.spawn(Vec2::new(4, 4), UnitKind::Humvee,  Team::Green);
    
    // Idea: Units that can demolish walls. Maybe Engineers?
    board.set_tile((3, 4).into(), Tile::Wall);
    board.set_tile((3, 5).into(), Tile::Wall);
    board.set_tile((6, 4).into(), Tile::Wall);
    board.set_tile((6, 5).into(), Tile::Wall);

    let mut game = Game {
        board:     board,
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
    
    while !graphics.root.window_closed() {
        graphics.root.set_default_background(BLACK);
        graphics.root.set_default_foreground(WHITE);
        draw(&mut graphics, &game);

        match game.state {
            PlayerState::Selecting => {
                graphics.root.set_alignment(TextAlignment::Center);
                graphics.root.print(12, 23, "=== Select ===");
                graphics.root.set_alignment(TextAlignment::Left);
            
                if let Some(unit) = game.board.get_unit(game.world_pos) {
                    graphics.root.print(1, 1, format!("HP {}/{}", unit.health, unit.health_max));
                }
            },

            PlayerState::Moving => {
                graphics.root.set_alignment(TextAlignment::Center);
                graphics.root.print(12, 23, "=== Move ===");
                graphics.root.print(12, 22, "Cancel (Esc)");
                graphics.root.set_alignment(TextAlignment::Left);
            
                if let Some(unit) = game.board.get_unit(game.selection.unwrap()) {
                    graphics.root.print(1, 1, format!("HP {}/{}", unit.health, unit.health_max));
                }

                if let Some(target) = game.board.get_unit(game.world_pos) {
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

            _ => {  }
        }

        if mouse.lbutton_pressed {
            println!("Mouse clicked {}", game.mouse_pos);

            let world_pos = game.mouse_pos.into_world(&graphics);

            // This particular section seems messy/fragile.
            match game.state {
                PlayerState::Selecting => {
                    if let Some(unit) = game.board.get_unit(world_pos) {
                        println!("Selected unit.");
                        game.selection = Some(world_pos);

                        let mut movement = Movement::new(world_pos, unit.speed);
                        movement_query(&game.board, &mut movement);

                        game.movement  = Some(movement);
                        game.state     = PlayerState::Moving;
                    }
                },

                PlayerState::Moving => {
                    println!("Attemping to move.");

                    let selection = game.selection.unwrap();

                    if let Some(movement) = &game.movement {
                        if movement.valid_tiles.contains(&world_pos) {
                            do_move(&mut game.board, Move::new(selection, world_pos));

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
    }
}