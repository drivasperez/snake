use super::{Materials, Position, Size};
use crate::direction::Direction;
use bevy::prelude::*;
use rand::prelude::random;
use std::time::Duration;

use super::Food;
use super::{ARENA_HEIGHT, ARENA_WIDTH};

struct SnakeHead {
    direction: Direction,
}

struct SnakeSegment;

#[derive(Default)]
struct SnakeSegments(Vec<Entity>);

#[derive(Default)]
struct LastTailPosition(Option<Position>);

struct MovementDuration(u64);

struct GrowthEvent;
struct GameOverEvent;

fn spawn_snake(
    mut commands: Commands,
    materials: Res<Materials>,
    mut segments: ResMut<SnakeSegments>,
) {
    let direction: Direction = random();
    let head_x = (ARENA_WIDTH / 2) as i32;
    let head_y = (ARENA_HEIGHT / 2) as i32;

    let (offset_x, offset_y) = match direction {
        Direction::Up => (0, -1),
        Direction::Right => (1, 0),
        Direction::Down => (0, 1),
        Direction::Left => (-1, 0),
    };

    let tail_position = Position {
        x: head_x + offset_x,
        y: head_y + offset_y,
    };

    segments.0 = vec![
        commands
            .spawn(SpriteComponents {
                material: materials.head_material.clone(),
                sprite: Sprite::new(Vec2::new(10.0, 10.0)),
                ..Default::default()
            })
            .with(SnakeHead {
                direction: random(),
            })
            .with(SnakeSegment)
            .with(Position {
                x: head_x,
                y: head_y,
            })
            .with(Size::square(0.8))
            .current_entity()
            .unwrap(),
        spawn_segment(&mut commands, &materials.segment_material, tail_position),
    ];
}

fn snake_movement(
    keyboard_input: Res<Input<KeyCode>>,
    snake_timer: ResMut<SnakeMoveTimer>,
    segments: ResMut<SnakeSegments>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_events: ResMut<Events<GameOverEvent>>,
    mut heads: Query<(Entity, &mut SnakeHead)>,
    mut positions: Query<&mut Position>,
) {
    if let Some((head_entity, mut head)) = heads.iter_mut().next() {
        let segment_positions: Vec<Position> = segments
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect();

        last_tail_position.0 = Some(*segment_positions.last().unwrap());

        let mut head_pos = positions.get_mut(head_entity).unwrap();
        let dir: Direction = if keyboard_input.pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Down) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::Up) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::Right) {
            Direction::Right
        } else {
            head.direction
        };

        if dir != head.direction.opposite() {
            head.direction = dir;
        }
        if !snake_timer.0.finished {
            return;
        }
        match &head.direction {
            Direction::Left => {
                head_pos.x -= 1;
            }
            Direction::Right => {
                head_pos.x += 1;
            }
            Direction::Up => {
                head_pos.y += 1;
            }
            Direction::Down => {
                head_pos.y -= 1;
            }
        }

        if head_pos.x < 0
            || head_pos.y < 0
            || head_pos.x as u32 >= ARENA_WIDTH
            || head_pos.y as u32 >= ARENA_HEIGHT
            || segment_positions.contains(&head_pos)
        {
            game_over_events.send(GameOverEvent);
        }

        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });
    }
}

fn snake_growth(
    mut commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    growth_events: Res<Events<GrowthEvent>>,
    mut segments: ResMut<SnakeSegments>,
    mut movement_timer: ResMut<SnakeMoveTimer>,
    mut movement_duration: ResMut<MovementDuration>,
    mut growth_reader: Local<EventReader<GrowthEvent>>,
    materials: Res<Materials>,
) {
    if growth_reader.iter(&growth_events).next().is_some() {
        segments.0.push(spawn_segment(
            &mut commands,
            &materials.segment_material,
            last_tail_position.0.unwrap(),
        ));

        if movement_duration.0 >= 20 {
            movement_duration.0 -= 10;
            movement_timer.0 = Timer::new(Duration::from_millis(movement_duration.0), true);
        }
    }
}

fn snake_eating(
    mut commands: Commands,
    snake_timer: ResMut<SnakeMoveTimer>,
    mut growth_events: ResMut<Events<GrowthEvent>>,
    food_positions: Query<With<Food, (Entity, &Position)>>,
    head_positions: Query<With<SnakeHead, &Position>>,
) {
    if !snake_timer.0.finished {
        return;
    }

    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.despawn(ent);
                growth_events.send(GrowthEvent);
            }
        }
    }
}

fn snake_timer(time: Res<Time>, mut snake_timer: ResMut<SnakeMoveTimer>) {
    snake_timer.0.tick(time.delta_seconds);
}

fn spawn_segment(
    commands: &mut Commands,
    material: &Handle<ColorMaterial>,
    position: Position,
) -> Entity {
    commands
        .spawn(SpriteComponents {
            material: material.clone(),
            ..Default::default()
        })
        .with(SnakeSegment)
        .with(position)
        .with(Size::square(0.65))
        .current_entity()
        .unwrap()
}

fn game_over(
    mut commands: Commands,
    mut reader: Local<EventReader<GameOverEvent>>,
    game_over_events: Res<Events<GameOverEvent>>,
    materials: Res<Materials>,
    segments_res: ResMut<SnakeSegments>,
    mut movement_timer: ResMut<SnakeMoveTimer>,
    food: Query<With<Food, Entity>>,
    segments: Query<With<SnakeSegment, Entity>>,
) {
    if reader.iter(&game_over_events).next().is_some() {
        for ent in food.iter().chain(segments.iter()) {
            commands.despawn(ent);
        }
        movement_timer.0 = Timer::new(Duration::from_millis(350), true);

        spawn_snake(commands, materials, segments_res);
    }
}

pub(crate) struct SnakeMoveTimer(Timer);

pub struct SnakePlugin;

impl Plugin for SnakePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(SnakeMoveTimer(Timer::new(Duration::from_millis(350), true)))
            .add_resource(SnakeSegments::default())
            .add_resource(LastTailPosition::default())
            .add_resource(MovementDuration(350));

        app.add_startup_system_to_stage("game_setup", spawn_snake.system())
            .add_system(snake_movement.system())
            .add_system(snake_timer.system())
            .add_system(snake_eating.system())
            .add_system(snake_growth.system())
            .add_system(game_over.system());

        app.add_event::<GrowthEvent>().add_event::<GameOverEvent>();
    }
}
