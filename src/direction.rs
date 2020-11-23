use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    pub(crate) fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Up => Self::Down,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
        }
    }
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0, 4) {
            0 => Direction::Left,
            1 => Direction::Up,
            2 => Direction::Right,
            _ => Direction::Down,
        }
    }
}
