use tcod::colors::*;
use crate::position::*;
use crate::board::{Traverse};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Team {
    Red,
    Blue,
    Green,
    Yellow,
    White
}

impl Team {
    pub fn color(&self) -> Color {
        match self {
            Team::Red    => RED,
            Team::Blue   => BLUE,
            Team::Green  => GREEN,
            Team::Yellow => YELLOW,
            Team::White  => WHITE
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Space {
    Ground,
    Water,
    Air
}

impl Space {
    pub fn can_traverse(&self, traverse: Traverse) -> bool {
        match (self, traverse) {
            (Space::Ground, Traverse::Ground) => true,
            (Space::Ground, Traverse::Water)  => false,
            (Space::Water,  Traverse::Ground) => false,
            (Space::Water,  Traverse::Water)  => true,
            (Space::Air,    Traverse::Ground) => true,
            (Space::Air,    Traverse::Water)  => true,
            (_, _)                            => false,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum UnitKind {
    Unknown,
    Engineer,
    Infantry,
    Humvee,
    Tank,
    Missile,
    Flag
}

struct UnitBuilder {
    kind:     UnitKind,
    team:     Team,
    name:     String,
    glyph:    char,
    space:    Space,
    health:   u32,
    damage:   u32,
    range:    u32,
    actions:  u32,
    position: Position
}

impl UnitBuilder {
    fn new() -> Self {
        UnitBuilder {
            kind:    UnitKind::Unknown,
            team:    Team::White,
            name:    String::from("No Name"),
            glyph:   '?',
            space:   Space::Ground,
            health:  1,
            damage:  1,
            range:   1,
            actions: 1,
            position: Position::default()
        }
    }

    fn with_kind(mut self, kind: UnitKind) -> Self {
        self.kind = kind;
        self
    }

    fn with_team(mut self, team: Team) -> Self {
        self.team = team;
        self
    }

    fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    fn with_glyph(mut self, glyph: char) -> Self {
        self.glyph = glyph;
        self
    }

    fn with_space(mut self, space: Space) -> Self {
        self.space = space;
        self
    }

    fn with_health(mut self, health: u32) -> Self {
        self.health = health;
        self
    }

    fn with_damage(mut self, damage: u32) -> Self {
        self.damage = damage;
        self
    }

    fn with_range(mut self, range: u32) -> Self {
        self.range = range;
        self
    }

    fn with_actions(mut self, actions: u32) -> Self {
        self.actions = actions;
        self
    }

    fn with_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    fn build(self) -> Unit {
        Unit {
            kind:        self.kind,
            team:        self.team,
            name:        self.name,
            glyph:       self.glyph,
            space:       self.space,
            health:      self.health,
            health_max:  self.health,
            damage:      self.damage,
            range:       self.range,
            actions:     self.actions,
            actions_max: self.actions,
            position:    self.position
        }
    }
}

#[derive(Debug)]
pub struct Unit {
    pub kind:        UnitKind,
    pub team:        Team,
    pub name:        String,
    pub glyph:       char,
    pub space:       Space,
    pub health:      u32,
    pub health_max:  u32,
    pub damage:      u32,
    pub range:       u32,
    pub actions:     u32,
    pub actions_max: u32,
    pub position:    Position
}

impl Unit {
    pub fn new(kind: UnitKind, team: Team, position: Position) -> Self {
        let mut builder = UnitBuilder::new()
            .with_kind(kind)
            .with_team(team)
            .with_position(position);
        
        match kind {
            UnitKind::Unknown => {
                panic!("Cannot create unit with kind 'Unknown.'");
            },

            UnitKind::Engineer => {
                builder = builder
                    .with_name(String::from("Engineer"))
                    .with_glyph('\u{0080}')
                    .with_space(Space::Ground)
                    .with_health(1)
                    .with_damage(1)
                    .with_range(1)
                    .with_actions(2)
            },

            UnitKind::Infantry => {
                builder = builder
                    .with_name(String::from("Infantry"))
                    .with_glyph('\u{0081}')
                    .with_space(Space::Ground)
                    .with_health(2)
                    .with_damage(1)
                    .with_range(1)
                    .with_actions(2)
            },

            UnitKind::Humvee => {
                builder = builder
                    .with_name(String::from("Humvee"))
                    .with_glyph('\u{0083}')
                    .with_space(Space::Ground)
                    .with_health(3)
                    .with_damage(1)
                    .with_range(1)
                    .with_actions(3)
            },

            UnitKind::Tank => {
                builder = builder
                    .with_name(String::from("Tank"))
                    .with_glyph('\u{0085}')
                    .with_space(Space::Ground)
                    .with_health(4)
                    .with_damage(2)
                    .with_range(3)
                    .with_actions(2)
            }

            UnitKind::Missile => {
                builder = builder
                    .with_name(String::from("Missile"))
                    .with_glyph('\u{0082}')
                    .with_space(Space::Air)
                    .with_health(3)
                    .with_damage(10)
                    .with_range(3)
                    .with_actions(3)
            },

            UnitKind::Flag => {
                builder = builder
                    .with_name(String::from("Flag"))
                    .with_glyph('\u{0084}')
                    .with_space(Space::Ground)
                    .with_health(1)
                    .with_damage(0)
                    .with_range(0)
                    .with_actions(1)
            }
        }

        builder.build()
    }
}