use crate::{
    camera::MainCamera,
    enemy::Enemy,
    upgrades::Upgrades,
    util::{polar_to_cartesian, AnimatedSprite, AnimatedSpriteData, flt_equal},
    world::{Wall, Tile},
    AppState, collision::{ParaLine, rect_to_lines},
};
use benimator::SpriteSheetAnimation;
use bevy::prelude::*;
use impacted::CollisionShape;
use std::f32::consts::PI;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(move_player)
                .with_system(detect_enemy_collision)
        );
    }
}

#[derive(Component)]
pub struct Player;

impl Player {
    pub const SIZE: f32 = 24.0;
    const VELOCITY: f32 = 500.0;
}

// Spawn the player in the given start location
// This function should only be called by the world plugin
pub fn spawn_player(
    mut commands: Commands,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    mut textures: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    upgrades: Res<Upgrades>,
    start_location: Vec2,
) {
    // Define player size
    let size = Vec2::splat(Player::SIZE);

    let transform = Transform {
        translation: start_location.extend(1.0),
        scale: if upgrades.has_upgrade(Upgrades::SHRINK) {
            // Half player scale if shrink upgrade is active
            Vec2::splat(0.5)
        } else {
            Vec2::ONE
        }
        .extend(1.0),
        ..Transform::default()
    };

    let collision_shape = if upgrades.has_upgrade(Upgrades::SHRINK) {
        CollisionShape::new_rectangle(size.x / 2.0, size.y / 2.0)
    } else {
        CollisionShape::new_rectangle(size.x, size.y)
    };

    // Spawn player
    commands
        .spawn_bundle(AnimatedSprite::new(
            &mut animations,
            &mut textures,
            &asset_server,
            AnimatedSpriteData {
                path: "bee.png".into(),
                frames: 6,
                size,
                transform,
                ..AnimatedSpriteData::default()
            },
        ))
        .insert(collision_shape)
        .insert(Player);
}

// TODO remove
fn window_to_world(
    window: &Window,
    camera: &Transform,
    position: &Vec2,
) -> Vec3 {
    let center = camera.translation.truncate();
    let half_width = (window.width() / 2.0) * camera.scale.x;
    let half_height = (window.height() / 2.0) * camera.scale.y;
    let left = center.x - half_width;
    let bottom = center.y - half_height;
    Vec3::new(
        left + position.x * camera.scale.x,
        bottom + position.y * camera.scale.y,
        0.0,  // I'm working in 2D
    )
}

fn move_player(
    windows: Res<Windows>,
    time: Res<Time>,
    upgrades: Res<Upgrades>,
    camera: Query<&Camera, With<MainCamera>>,
    mut transform: Query<&mut Transform, (With<Player>, Without<MainCamera>)>,
    walls: Query<&Transform, (With<Wall>, Without<Player>)>,
    // TODO remove
    mut dbg_lines: ResMut<bevy_prototype_debug_lines::DebugLines>,
    cam_trans: Query<&Transform, (With<MainCamera>, Without<Wall>)>
) {
    let camera = camera.single();
    let window = windows.get(camera.window).unwrap();
    // Some(_) if the cursor is in the window
    if let Some(cursor_pos) = window.cursor_position() {
        let relative_pos = Vec2::new(
            cursor_pos.x - window.width() / 2.,
            cursor_pos.y - window.height() / 2.,
        );
        let velocity_angle = relative_pos.y.atan2(relative_pos.x);
        let magnitude_cap = window.width().min(window.height()) / 4.;
        // between 0 and 1
        let velocity_scale = relative_pos.length().min(magnitude_cap) / magnitude_cap;

        let mut velocity = polar_to_cartesian(velocity_angle, velocity_scale * Player::VELOCITY)
            * time.delta_seconds()
            * if upgrades.has_upgrade(Upgrades::DOUBLE_SPEED) {
                // Double velocity if player has double speed upgrade
                2.0
            } else {
                1.0
            };

        let mut transform = transform.single_mut();
        transform.rotation = Quat::from_rotation_z(velocity_angle - PI / 2.0);
        let player_normal = ParaLine::new(
            // from the front of the bee...
            transform.translation.truncate() + polar_to_cartesian(velocity_angle, Player::SIZE / 2.),
            // to the place where it's going to go
            velocity
        );
        // TODO remove
        let mut line_f = |a: Vec2, b: Vec2| {
            let cam = cam_trans.single();
            dbg_lines.line(a.extend(0.), b.extend(0.), 0.1);
        };
        line_f(player_normal.p, player_normal.p + player_normal.v);
        for wall in walls.iter() {
            let wall_size = Vec2::splat(Tile::SIZE);
            let wall_lines = rect_to_lines(wall.translation.truncate() - wall_size/2., wall_size);
            let collide = wall_lines
                .into_iter()
                .filter_map(|wall_line| player_normal.intersect(&wall_line).map(|t| (t, wall_line)))
                .reduce(|acc, next| if acc.0 < next.0 {
                    acc
                } else {
                    next
                });
            if let Some((t, collide_line)) = collide {
                println!("collide! collide_line = {collide_line:?}, t = {}, prev velocity = {}", t, velocity);
                // Shorten it so it doesn't collide anymore
                let short_v = velocity * t; // This brings it right to the wall
                let mut rest_v = velocity * (1.-t); // This is the leftover velocity
                if flt_equal(collide_line.v.x, 0.) {
                    println!("vert collide");
                    // Line is vertical, remove horizontal velocity past this point
                    rest_v.x = 0.;
                    velocity.x = 0.;
                } else {
                    println!("horiz collide");
                    // Line is horizontal, remove vertical velocity
                    rest_v.y = 0.;
                    velocity.y = 0.;
                }
                // Shouldn't collide anymore
                // velocity = short_v + rest_v;
                println!("new velocity = {}", velocity);
                // That's probably all we need, right?
                break;
            }
        }
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;
    }
}

fn detect_enemy_collision(
    mut state: ResMut<State<AppState>>,
    enemies: Query<&CollisionShape, (With<Enemy>, Changed<CollisionShape>)>,
    player: Query<&CollisionShape, With<Player>>,
) {
    let player = player.single();
    for enemy in enemies.iter() {
        if player.is_collided_with(enemy) {
            dbg!(&enemy);
            println!("Player has collided with enemy");
            state.set(AppState::Death).unwrap();
            return;
        }
    }
}
