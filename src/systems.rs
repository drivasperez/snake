use std::time::Duration;

use bevy::prelude::*;
use rand::prelude::*;

use crate::{
    components::{
        Direction, Food, FoodCount, FoodSpawnTimer, GameEvent, LastTailPosition, LifeSpan,
        Materials, MovementDuration, Position, Size, SnakeHead, SnakeMoveTimer, SnakeSegment,
        SnakeSegments,
    },
    ARENA_HEIGHT, ARENA_WIDTH, FOOD_LIFE_SPAN_MS,
};

pub fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut sprite) in q.iter_mut() {
        sprite.size = Vec2::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
        );
    }
}

pub fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    let convert = |pos, bound_window, bound_game| {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    };

    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

pub fn spawn_snake(
    mut commands: Commands,
    materials: Res<Materials>,
    mut segments: ResMut<SnakeSegments>,
) {
    let direction: Direction = random();
    let head_x = (ARENA_WIDTH / 2) as i32;
    let head_y = (ARENA_HEIGHT / 2) as i32;

    let (offset_x, offset_y) = match direction {
        Direction::Up => (0, -1),
        Direction::Right => (-1, 0),
        Direction::Down => (0, 1),
        Direction::Left => (1, 0),
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
            .with(SnakeHead { direction })
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

pub fn snake_movement(
    keyboard_input: Res<Input<KeyCode>>,
    snake_timer: ResMut<SnakeMoveTimer>,
    segments: ResMut<SnakeSegments>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_events: ResMut<Events<GameEvent>>,
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
        let dir: Direction =
            if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
                Direction::Left
            } else if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
                Direction::Down
            } else if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
                Direction::Up
            } else if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
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

        if head_pos.x < 0 {
            head_pos.x = (ARENA_WIDTH - 1) as i32;
        }
        if head_pos.y < 0 {
            head_pos.y = (ARENA_HEIGHT - 1) as i32;
        }
        if head_pos.x >= ARENA_WIDTH as i32 {
            head_pos.x = 0;
        }
        if head_pos.y >= ARENA_HEIGHT as i32 {
            head_pos.y = 0;
        }

        if segment_positions.contains(&head_pos) {
            game_events.send(GameEvent::GameOver);
        }

        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });
    }
}

pub fn snake_growth(
    mut commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    growth_events: Res<Events<GameEvent>>,
    mut segments: ResMut<SnakeSegments>,
    mut movement_timer: ResMut<SnakeMoveTimer>,
    mut movement_duration: ResMut<MovementDuration>,
    mut growth_reader: Local<EventReader<GameEvent>>,
    materials: Res<Materials>,
) {
    if let Some(GameEvent::Growth) = growth_reader.iter(&growth_events).next() {
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

pub fn snake_eating(
    mut commands: Commands,
    snake_timer: ResMut<SnakeMoveTimer>,
    mut growth_events: ResMut<Events<GameEvent>>,
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
                growth_events.send(GameEvent::Growth);
            }
        }
    }
}

pub fn snake_timer(time: Res<Time>, mut snake_timer: ResMut<SnakeMoveTimer>) {
    snake_timer.0.tick(time.delta_seconds);
}

pub fn spawn_segment(
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

pub fn game_over(
    mut commands: Commands,
    mut reader: Local<EventReader<GameEvent>>,
    events: Res<Events<GameEvent>>,
    materials: Res<Materials>,
    segments_res: ResMut<SnakeSegments>,
    mut movement_timer: ResMut<SnakeMoveTimer>,
    food: Query<With<Food, Entity>>,
    segments: Query<With<SnakeSegment, Entity>>,
) {
    if let Some(GameEvent::GameOver) = reader.iter(&events).next() {
        for ent in food.iter().chain(segments.iter()) {
            commands.despawn(ent);
        }
        movement_timer.0 = Timer::new(Duration::from_millis(350), true);

        spawn_snake(commands, materials, segments_res);
    }
}

pub fn food_spawner(
    mut commands: Commands,
    materials: Res<Materials>,
    time: Res<Time>,
    mut events: ResMut<Events<GameEvent>>,
    mut timer: Local<FoodSpawnTimer>,
    live_food: Res<FoodCount>,
    snake_positions: Query<With<SnakeSegment, &Position>>,
) {
    timer.0.tick(time.delta_seconds);

    let banned_positions: Vec<&Position> = snake_positions.iter().collect();
    let positions: Vec<Position> = (0..ARENA_WIDTH as i32)
        .flat_map(|x| {
            (0..ARENA_HEIGHT as i32)
                .map(|y| Position { x, y })
                .collect::<Vec<Position>>()
        })
        .filter(|pos| !banned_positions.contains(&pos))
        .collect();

    let position = *positions.choose(&mut rand::thread_rng()).unwrap();

    if timer.0.finished && live_food.0 <= 10 {
        commands
            .spawn(SpriteComponents {
                material: materials.food_material.clone(),
                ..Default::default()
            })
            .with(Food)
            .with(LifeSpan(Timer::new(
                Duration::from_millis(FOOD_LIFE_SPAN_MS),
                false,
            )))
            .with(position)
            .with(Size::square(0.8));

        events.send(GameEvent::SpawnedFood);
    }
}

pub fn food_despawner(
    mut commands: Commands,
    time: Res<Time>,
    mut events: ResMut<Events<GameEvent>>,
    mut food: Query<With<Food, (Entity, &mut LifeSpan)>>,
) {
    for (ent, mut age) in food.iter_mut() {
        age.0.tick(time.delta_seconds);
        if age.0.finished {
            commands.despawn(ent);
            events.send(GameEvent::FoodRotted);
        }
    }
}

pub fn food_count(
    mut food_count: ResMut<FoodCount>,
    events: Res<Events<GameEvent>>,
    mut reader: Local<EventReader<GameEvent>>,
) {
    for event in reader.iter(&events) {
        use GameEvent::*;
        match event {
            SpawnedFood => food_count.0 += 1,
            Growth | FoodRotted => {
                if food_count.0 > 0 {
                    food_count.0 -= 1
                }
            }
            GameOver => food_count.0 = 0,
        }
    }
}
