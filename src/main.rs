use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use std::time::Duration;
use systems::{
    food_count, food_despawner, food_spawner, game_over, position_translation, size_scaling,
    snake_eating, snake_growth, snake_movement, snake_timer, spawn_snake,
};

use components::{
    FoodCount, GameEvent, LastTailPosition, Materials, MovementDuration, SnakeMoveTimer,
    SnakeSegments,
};

mod components;
mod systems;

const ARENA_WIDTH: u32 = 15;
const ARENA_HEIGHT: u32 = 15;
const FOOD_SPAWN_RATE: u64 = 2000;
const FOOD_LIFE_SPAN_MS: u64 = 5000;

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let head_handle = asset_server.load("segment.png");
    let segment_handle = asset_server.load("segment.png");
    commands
        .spawn(Camera2dComponents::default())
        .insert_resource(Materials {
            head_material: materials.add(head_handle.into()),
            segment_material: materials.add(segment_handle.into()),
            food_material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
        });
}

fn main() {
    let mut app = App::build();

    app.add_resource(WindowDescriptor {
        title: String::from("Snake!"),
        width: 800,
        height: 800,
        ..Default::default()
    })
    .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
    .add_resource(FoodCount(0))
    .add_resource(SnakeMoveTimer(Timer::new(Duration::from_millis(350), true)))
    .add_resource(SnakeSegments::default())
    .add_resource(LastTailPosition::default())
    .add_resource(MovementDuration(350));

    app.add_startup_system(setup.system())
        .add_startup_stage("game_setup");

    app.add_startup_system_to_stage("game_setup", spawn_snake.system())
        .add_system(snake_movement.system())
        .add_system(snake_timer.system())
        .add_system(snake_eating.system())
        .add_system(snake_growth.system())
        .add_system(game_over.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .add_system(food_spawner.system())
        .add_system(food_despawner.system())
        .add_system(food_count.system());

    app.add_event::<GameEvent>();
    app.add_plugins(DefaultPlugins);

    app.run();
}
