extern crate tcod;
extern crate generational_arena;

use tcod::console::*;
use tcod::colors::*;

use generational_arena::Arena;
use generational_arena::Index as EntityIndex;

use std::collections::VecDeque;

mod entity;
mod position;
mod board;
mod action_circle;
mod input;
mod utilities;
mod menu;

use entity::*;
use position::*;
use board::*;
use action_circle::*;
use input::*;
use utilities::*;
use menu::*;

#[derive(Debug, PartialEq, Copy, Clone)]
enum PlayerState {
    Selecting,
    Controlling(EntityIndex),
    GameOver
}

#[derive(Debug)]
struct Game {
    player_state: PlayerState,
    player:       Team,
    board:        Board,

    units:        Arena<Unit>,
    damage_queue: Vec<DamageAtPos>,
}

#[derive(Debug)]
pub struct SpawnData {
    kind:     UnitKind,
    team:     Team,
    position: Position
}

impl SpawnData {
    fn new(kind: UnitKind, team: Team, position: Position) -> Self {
        SpawnData {
            kind,
            team,
            position
        }
    }
}

#[derive(Debug)]
pub enum SpawnError {
    PositionOutOfBounds,
    PositionOccupied
}

impl Game {
    fn spawn(&mut self, data: SpawnData) -> Result<EntityIndex, SpawnError> {
        let index = self.board.to_index(data.position).ok_or(SpawnError::PositionOutOfBounds)?;
        
        if self.board.entity_at(data.position).is_some() {
            return Err(SpawnError::PositionOccupied);
        }

        let unit = Unit::new(data.kind, data.team, data.position);
        let entity = self.units.insert(unit);

        self.board.entities[index] = Some(entity);

        Ok(entity)
    }

    fn next_turn(&mut self) -> bool {
        let current_team  = self.player;
        let mut next_team = get_next_team(current_team);

        let mut next_turn_valid = false;

        while !next_turn_valid && current_team != next_team {
            println!("Checking {:?}...", next_team);

            for (_, unit) in &self.units {
                if unit.team == next_team {
                    next_turn_valid = true;
                    break;
                }
            }

            if !next_turn_valid {
                next_team = get_next_team(next_team);
            }
        }

        if next_turn_valid {
            self.player       = next_team;
            self.player_state = PlayerState::Selecting;
            
            for (_, unit) in &mut self.units {
                if unit.team == self.player {
                    unit.actions = unit.actions_max;
                }
            }

            println!("{:?}'s turn!", self.player);

            true
        } else {
            self.player_state = PlayerState::GameOver;
            
            println!("Game over!");

            false
        }
    }
}

pub struct Graphics {
    pub root:         Root,
    pub board:        Offscreen,
    pub board_offset: Position
}

#[derive(Debug)]
pub struct IntentToMove {
    pub entity: EntityIndex,
    pub to:     Position
}

#[derive(Debug)]
enum MoveError {
    UnitInvalid,
    UnitExhausted,
    TerrainIncompatible,
    DestinationOccupied,
    DestinationUnreachable
}

fn move_unit(game: &mut Game, intent: IntentToMove) -> Result<(), MoveError> {
    let mut unit = game.units.get_mut(intent.entity).ok_or(MoveError::UnitInvalid)?;

    if unit.actions == 0 {
        return Err(MoveError::UnitExhausted);
    }

    if game.board.entity_at(intent.to).is_some() {
        return Err(MoveError::DestinationOccupied);
    }

    let action_circle = ActionCircle::new(unit.position, unit.actions, Some(unit.space), &game.board);
    if !action_circle.contains(intent.to) {
        return Err(MoveError::DestinationUnreachable);
    }

    let index_from = game.board.to_index_unchecked(unit.position);
    let index_to   = game.board.to_index_unchecked(intent.to);

    let tile = game.board.tile_at(intent.to).unwrap();
    if !unit.space.can_traverse(tile.traverse()) {
        return Err(MoveError::TerrainIncompatible);
    }
    
    game.board.entities[index_to]   = game.board.entities[index_from];
    game.board.entities[index_from] = None;

    unit.position = intent.to;

    unit.actions -= action_circle.cost_to(intent.to).unwrap();

    Ok(())
}

#[derive(Debug)]
struct IntentToAttack {
    entity:        EntityIndex,
    target_entity: EntityIndex
}

#[derive(Debug)]
enum AttackError {
    UnitInvalid,
    UnitExhausted,
    TargetInvalid,
    TargetFriendly,
    TargetOutOfRange
}

#[derive(Debug)]
struct DamageAtPos {
    at:     Position,
    amount: u32
}

impl DamageAtPos {
    fn new(at: Position, amount: u32) -> Self {
        DamageAtPos {
            at,
            amount
        }
    }
}

fn attack_with_unit(game: &mut Game, intent: IntentToAttack) -> Result<(), AttackError> {
    if intent.entity == intent.target_entity {
        return Err(AttackError::TargetFriendly);
    }

    let (unit, target) = game.units.get2_mut(intent.entity, intent.target_entity);
    
    let mut unit = unit.ok_or(AttackError::UnitInvalid)?;
    let target   = target.ok_or(AttackError::TargetInvalid)?;

    let position        = unit.position;
    let target_position = target.position;
    let action_circle   = ActionCircle::new(position, unit.range, Some(unit.space), &game.board);
    
    if !action_circle.contains(target_position) {
        return Err(AttackError::TargetOutOfRange);
    }
    
    if unit.team == target.team {
        return Err(AttackError::TargetFriendly);
    }

    if unit.actions == 0 {
        return Err(AttackError::UnitExhausted);
    }

    let damage = DamageAtPos::new(target_position, unit.damage);
    game.damage_queue.push(damage);

    if unit.kind == UnitKind::Missile {
        unit.health = 0;
        let explosion_radius = target_position.radius(2);
        for position in explosion_radius {
            let damage = DamageAtPos::new(position, unit.damage);
            game.damage_queue.push(damage);
        }
    }

    unit.actions = 0;
    
    Ok(())
}

fn draw(game: &Game, graphics: &mut Graphics, input: &Input) {
    let board = &game.board;
    
    graphics.root.clear();

    // Draw tiles.
    for y in 0..board.height() {
        for x in 0..board.width() {
            let position   = Position::new(x as i32, y as i32);
            let tile       = board.tile_at(position).unwrap();
            let fore_color = tile.fore_color();
            let back_color = tile.back_color();
            let glyph      = tile.glyph();

            graphics.board.put_char_ex(
                x as i32,
                y as i32,
                glyph,
                fore_color,
                back_color
            );
        }
    }

    // Draw entities
    for (_, unit) in &game.units {
        graphics.board.set_char(
            unit.position.x,
            unit.position.y,
            unit.glyph
        );

        if unit.actions != 0 {
            graphics.board.set_char_foreground(
                unit.position.x,
                unit.position.y,
                unit.team.color()
            );
        } else {
            graphics.board.set_char_foreground(
                unit.position.x,
                unit.position.y,
                darken(unit.team.color())
            );
        }
    }

    // Highlight the selected entity
    if let PlayerState::Controlling(entity) = game.player_state {
        let unit = game.units.get(entity).unwrap();

        let fore_color = graphics.board.get_char_foreground(unit.position.x, unit.position.y);
        let back_color = graphics.board.get_char_background(unit.position.x, unit.position.y);

        graphics.board.set_char_foreground(unit.position.x, unit.position.y, back_color);
        graphics.board.set_char_background(unit.position.x, unit.position.y, fore_color, BackgroundFlag::Set);

        if unit.actions != 0 {
            let action_circle = ActionCircle::new(unit.position, unit.range, Some(unit.space), &game.board);
            for (position, _) in action_circle {    
                graphics.board.set_char_background(
                    position.x,
                    position.y,
                    DARKEST_RED,
                    BackgroundFlag::Set
                );
            }
        }

        let action_circle = ActionCircle::new(unit.position, unit.actions, Some(unit.space), &game.board);
        for (position, _) in action_circle {
            graphics.board.set_char_background(
                position.x,
                position.y,
                DARKEST_GREY,
                BackgroundFlag::Add
            );
        }
    }

    // Highlight mouse position
    let world_pos = input.mouse().world_pos;
    if let Some(tile) = game.board.tile_at(world_pos) {
        if !tile.is_wall() {
            graphics.board.set_char_background(
                world_pos.x,
                world_pos.y,
                DARKER_YELLOW,
                BackgroundFlag::Set
            );
        }
    }



    blit(
        &graphics.board, 
        (0, 0), 
        (0, 0), 
        &mut graphics.root, 
        (graphics.board_offset.x, graphics.board_offset.y), 
        1.0, 
        1.0
    );



    // =========== Draw UI =========== //
    
    // Turn label.
    graphics.root.set_default_foreground(game.player.color());
    graphics.root.print(1, graphics.root.height() - 3, format!("{:?}'s turn", game.player));
    graphics.root.set_default_foreground(WHITE);

    // Arrow before the current-state label.
    graphics.root.set_char(
        1,
        graphics.root.height() - 2,
        '\u{001A}'
    );

    graphics.root.set_char_foreground(
        1,
        graphics.root.height() - 2,
        GREY
    );

    // Current-state label.
    match game.player_state {
        PlayerState::Selecting => {
            graphics.root.print(
                2,
                graphics.root.height() - 2,
                "Selecting"
            );
        },

        PlayerState::Controlling(_) => {
            graphics.root.print(
                2,
                graphics.root.height() - 2,
                "Moving"
            );
        },

        _ => {

        }
    }

    // Health and Action Points.
    if let Some(entity) = game.board.entity_at(world_pos) {
        if let Some(unit) = game.units.get(entity) {
            graphics.root.set_default_foreground(unit.team.color());
            graphics.root.print(
                1,
                1,
                format!("{}", unit.name)
            );       
            graphics.root.set_default_foreground(WHITE);

            graphics.root.print(
                1,
                2,
                format!("HP {}/{} | AP {}/{}", unit.health, unit.health_max, unit.actions, unit.actions_max)
            )
        }
    }

    graphics.root.flush();
}

fn read_input(game: &mut Game, graphics: &mut Graphics, input: &mut Input) {
    input.update(game.board.size(), graphics.board_offset);

    let world_pos = input.mouse().world_pos;

    if input.key(KeyCode::O).down {
        if !game.next_turn() {
            return;
        }
    }

    if input.button(MouseButton::Right).down {
        spawn_menu(game, graphics, input, world_pos);
        return;
    }

    match game.player_state {
        PlayerState::Selecting => {
            if input.button(MouseButton::Left).down {
                if let Some(entity) = game.board.entity_at(world_pos) {
                    let unit = game.units.get(entity).unwrap();
                    if unit.team == game.player {
                        game.player_state = PlayerState::Controlling(entity);
                    }
                }
            }
        },

        PlayerState::Controlling(entity) => {
            if input.key(KeyCode::Escape).down {
                game.player_state = PlayerState::Selecting;
                return;
            }

            if input.button(MouseButton::Left).down {
                if let Some(target_entity) = game.board.entity_at(world_pos) {
                    let intent = IntentToAttack {
                        entity,
                        target_entity
                    };

                    let result = attack_with_unit(game, intent);
                    match result {
                        Ok(()) => {
                            println!("[Attack] Success");
                            let unit = game.units.get(entity).unwrap();
                            if unit.actions == 0 {
                                game.player_state = PlayerState::Selecting;
                            }
                        },

                        Err(error) => {
                            println!("[Attack] Failure ({:?})", error);
                            match error {
                                AttackError::UnitInvalid |
                                AttackError::UnitExhausted => {
                                    game.player_state = PlayerState::Selecting;
                                },

                                _ => {

                                }
                            }
                        }
                    }
                } else {
                    let intent = IntentToMove {
                        entity,
                        to: world_pos
                    };
    
                    let result = move_unit(game, intent);
                    match result {
                        Ok(()) => {
                            println!("[Move] Success");
                            let unit = game.units.get(entity).unwrap();
                            if unit.actions == 0 {
                                game.player_state = PlayerState::Selecting;
                            }
                        },
    
                        Err(error) => {
                            println!("[Move] Failure ({:?})", error);
                            match error {
                                MoveError::UnitInvalid |
                                MoveError::UnitExhausted => {
                                    game.player_state = PlayerState::Selecting;
                                }

                                _ => {

                                }
                            }
                        }
                    }
                }
            }
        },
        
        _ => {

        }
    }
}

fn spawn_menu(game: &mut Game, graphics: &mut Graphics, input: &mut Input, at: Position) {
    let builder = MenuBuilder::new()
        .with_prompt(String::from("Spawn/Unit"))
        .with_option(String::from("Engineer"), UnitKind::Engineer)
        .with_option(String::from("Infantry"), UnitKind::Infantry)
        .with_option(String::from("Humvee"),   UnitKind::Humvee)
        .with_option(String::from("Tank"),     UnitKind::Tank)
        .with_option(String::from("Missile"),  UnitKind::Missile)
        .with_option(String::from("Flag"),     UnitKind::Flag);

    let menu = builder.build();

    let kind: UnitKind;

    loop {
        input.update(game.board.size(), graphics.board_offset);
        let result = menu.show(graphics, input);
        match result {
            MenuResult::Selected(item) => {
                kind = item;
                break;
            }

            MenuResult::NoResponse => {

            }

            MenuResult::Cancel => {
                return;
            }
        }
    }



    let builder = MenuBuilder::new()
        .with_prompt(String::from("Spawn/Unit/Team"))
        .with_option(String::from("Red"),    Team::Red)
        .with_option(String::from("Blue"),   Team::Blue)
        .with_option(String::from("Green"),  Team::Green)
        .with_option(String::from("Yellow"), Team::Yellow)
        .with_option(String::from("White"),  Team::White);

    let menu = builder.build();

    let team: Team;

    loop {
        input.update(game.board.size(), graphics.board_offset);
        let result = menu.show(graphics, input);
        match result {
            MenuResult::Selected(item) => {
                team = item;
                break;
            }

            MenuResult::NoResponse => {

            }

            MenuResult::Cancel => {
                return;
            }
        }
    }

    let result = game.spawn(SpawnData::new(kind, team, at));
    match result {
        Ok(entity) => println!("[Spawn] Success (entity={:?})", entity),
        Err(error) => println!("[Spawn] Failure ({:?})", error)
    }
}

fn bring_out_your_dead(game: &mut Game) {
    let mut killed: Vec<EntityIndex> = Vec::new();
    for damage in &game.damage_queue {
        if let Some(entity) = game.board.entity_at(damage.at) {
            let unit = game.units.get_mut(entity).unwrap();
            unit.health -= damage.amount.min(unit.health);
            
            if !killed.contains(&entity) && unit.health == 0 {
                killed.push(entity);
            }
        } 
    }

    game.damage_queue.clear();

    for entity in killed {
        for i in 0..game.board.entities.len() {
            if game.board.entities[i] == Some(entity) {
                game.board.entities[i] = None;
            }
        }

        game.units.remove(entity);
    }
}

fn main() {
    println!("Hello, world!");
    
    let mut graphics = Graphics {
        root: Root::initializer()
                .size(24, 20)
                .title("A Starless Void")
                .font("res/Font 16x16 Extended.png", FontLayout::AsciiInRow)
                .init(),

        board:        Offscreen::new(10, 10),
        board_offset: Position::new(7, 5)
    };
    
    let mut input = Input::new();

    let mut game = Game {
        player_state: PlayerState::Selecting,
        player:       Team::White,
        damage_queue: Vec::new(),
        units:        Arena::new(),
        board:        Board::new(Dimension::new(10, 10))
    };

    game.spawn(SpawnData::new(UnitKind::Engineer, Team::Red,    Position::new(2, 2))).unwrap();
    game.spawn(SpawnData::new(UnitKind::Infantry, Team::Blue,   Position::new(4, 1))).unwrap();
    game.spawn(SpawnData::new(UnitKind::Infantry, Team::Blue,   Position::new(5, 2))).unwrap();
    game.spawn(SpawnData::new(UnitKind::Humvee,   Team::Green,  Position::new(2, 7))).unwrap();
    game.spawn(SpawnData::new(UnitKind::Tank,     Team::Yellow, Position::new(4, 6))).unwrap();

    if !game.next_turn() {
        println!("Could not start. No units on the battlefield.");
        return;
    }

    while !graphics.root.window_closed() {
        draw(&game, &mut graphics, &input);
        read_input(&mut game, &mut graphics, &mut input);
        
        bring_out_your_dead(&mut game);

        if game.player_state == PlayerState::GameOver {
            break;
        }
    }

    println!("Goodbyte, world!");
}