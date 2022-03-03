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

fn move_player(
    windows: Res<Windows>,
    time: Res<Time>,
    upgrades: Res<Upgrades>,
    camera: Query<&Camera, With<MainCamera>>,
    mut transform: Query<&mut Transform, (With<Player>, Without<MainCamera>)>,
    walls: Query<&Transform, (With<Wall>, Without<Player>)>,
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

        let velocity = polar_to_cartesian(velocity_angle, velocity_scale * Player::VELOCITY)
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
        let mut new_x = transform.translation.x + velocity.x;
        let mut new_y = transform.translation.y + velocity.y;
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
                println!("about to collide in {t} frames");
                // We want to offset it so it won't collide anymore.
                // We need to check which corner (top left or top right) of the bee is going to
                // collide first, so we know how to offset it.
                // Don't ask me how I came up with this.
                let mut top_right_corner_collide = (velocity.x >= 0.) ^ (velocity.y >= 0.);
                let vert_collide = flt_equal(collide_line.v.x, 0.);
                if vert_collide {
                    println!("vert collide");
                    top_right_corner_collide = !top_right_corner_collide;
                } else {
                    println!("horiz collide");
                }
                dbg!(&top_right_corner_collide);
                // (x-basis, y-basis)
                let bee_basis = (polar_to_cartesian(velocity_angle - PI / 2., 1.), polar_to_cartesian(velocity_angle, 1.));
                let corner_x_offset = if top_right_corner_collide {
                    Player::SIZE/2.
                } else {
                    // top left corner
                    -Player::SIZE/2.
                };
                let corner_offset = corner_x_offset * bee_basis.0 + Player::SIZE/2. * bee_basis.1;
                let corner_pos = transform.translation.truncate() + corner_offset;
                // We want to set the corner such that it 'just touches' the wall.
                // Hence (current corner) + (push) touches wall.
                let push: Vec2 = if vert_collide { 
                    (collide_line.p.x - corner_pos.x, 0.)
                } else { 
                    (0., collide_line.p.y - corner_pos.y)
                }.into();
                dbg!(&corner_offset, &push);
                new_x += push.x - velocity.x;
                new_y += push.y - velocity.y;
                break;
            }
        }
        transform.translation.x = new_x;
        transform.translation.y = new_y;
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
