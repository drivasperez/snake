use bevy::prelude::*;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::time::Duration;

use crate::FOOD_SPAWN_RATE;
#[derive(Debug)]
pub enum GameEvent {
    Growth,
    GameOver,
    SpawnedFood,
    FoodRotted,
}

pub struct Materials {
    pub head_material: Handle<ColorMaterial>,
    pub segment_material: Handle<ColorMaterial>,
    pub food_material: Handle<ColorMaterial>,
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

pub struct SnakeHead {
    pub direction: Direction,
}

pub struct SnakeSegment;

#[derive(Default)]
pub struct SnakeSegments(pub Vec<Entity>);

#[derive(Default)]
pub struct LastTailPosition(pub Option<Position>);

pub struct MovementDuration(pub u64);

pub struct SnakeMoveTimer(pub Timer);

pub struct Food;
pub struct FoodCount(pub u32);
pub struct LifeSpan(pub Timer);

pub struct FoodSpawnTimer(pub Timer);

impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(FOOD_SPAWN_RATE), true))
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    pub fn opposite(self) -> Self {
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
