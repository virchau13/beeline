mod camera;
mod enemy;
mod menu;
mod player;
mod pursue;
mod util;

use benimator::AnimationPlugin;
use bevy::prelude::*;

use camera::CameraPlugin;
use enemy::EnemyPlugin;
use menu::MenuPlugin;
use player::PlayerPlugin;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum AppState {
    Menu,
    Game,
}

fn despawn_all(mut commands: Commands, entities: Query<Entity>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state(AppState::Menu)
        .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(despawn_all))
        .add_system_set(SystemSet::on_exit(AppState::Game).with_system(despawn_all))
        .add_plugin(MenuPlugin)
        .add_plugin(AnimationPlugin::default())
        .add_plugin(CameraPlugin)
        .add_plugin(EnemyPlugin)
        .add_plugin(PlayerPlugin)
        .run();
}
