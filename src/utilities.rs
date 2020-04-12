use tcod::Color;
use crate::Team;

pub fn darken(color: Color) -> Color {
    let (hue, saturation, value) = color.hsv();
    Color::new_from_hsv(hue, saturation, value * 0.4)
}

pub fn get_next_team(team: Team) -> Team {
    match team {
        Team::Red    => Team::Blue,
        Team::Blue   => Team::Green,
        Team::Green  => Team::Yellow,
        Team::Yellow => Team::White,
        Team::White  => Team::Red
    }
}