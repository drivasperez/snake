use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use rand::prelude::random;
use std::time::Duration;

mod direction;
mod snake;

use snake::SnakePlugin;

const ARENA_WIDTH: u32 = 25;
const ARENA_HEIGHT: u32 = 25;
const FOOD_SPAWN_RATE: u64 = 2000;
const FOOD_LIFE_SPAN_MS: u64 = 5000;

struct Materials {
    head_material: Handle<ColorMaterial>,
    segment_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>,
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands
        .spawn(Camera2dComponents::default())
        .insert_resource(Materials {
            head_material: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
            segment_material: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
            food_material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
        });
}

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: String::from("Snake!"),
            width: 800,
            height: 800,
            ..Default::default()
        })
        .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_resource(FoodCount(0))
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup")
        .add_plugin(SnakePlugin)
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .add_system(food_spawner.system())
        .add_system(food_despawner.system())
        .add_plugins(DefaultPlugins)
        .run();
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut sprite) in q.iter_mut() {
        sprite.size = Vec2::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
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

struct Food;
struct FoodCount(u32);
struct LifeSpan(Timer);

struct FoodSpawnTimer(Timer);

impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(FOOD_SPAWN_RATE), true))
    }
}

fn food_spawner(
    mut commands: Commands,
    materials: Res<Materials>,
    time: Res<Time>,
    mut timer: Local<FoodSpawnTimer>,
    mut live_food: ResMut<FoodCount>,
) {
    timer.0.tick(time.delta_seconds);
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
            .with(Position {
                x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
                y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
            })
            .with(Size::square(0.8));

        live_food.0 += 1;
    }
}

fn food_despawner(
    mut commands: Commands,
    time: Res<Time>,
    mut food_count: ResMut<FoodCount>,
    mut food: Query<With<Food, (Entity, &mut LifeSpan)>>,
) {
    for (ent, mut age) in food.iter_mut() {
        age.0.tick(time.delta_seconds);
        if age.0.finished {
            commands.despawn(ent);
            food_count.0 -= 1;
        }
    }
}
