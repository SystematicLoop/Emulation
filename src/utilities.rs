use tcod::{Console, Color, BackgroundFlag};
use crate::{Team, Position};

pub fn darken(color: Color) -> Color {
    let (hue, saturation, value) = color.hsv();
    Color::new_from_hsv(hue, saturation, value * 0.4)
}

pub fn get_next_team(team: Team) -> Team {
    match team {
        Team::Red     => Team::Blue,
        Team::Blue    => Team::Green,
        Team::Green   => Team::Yellow,
        Team::Yellow  => Team::Cyan,
        Team::Cyan    => Team::Orange,
        Team::Orange  => Team::Magenta,
        Team::Magenta => Team::White,
        Team::White   => Team::Red
    }
}

pub fn invert_cell(console: &mut dyn Console, position: Position) {
    let fore_color = console.get_char_foreground(position.x, position.y);
    let back_color = console.get_char_background(position.x, position.y);

    console.set_char_foreground(position.x, position.y, back_color);
    console.set_char_background(position.x, position.y, fore_color, BackgroundFlag::Set);
}