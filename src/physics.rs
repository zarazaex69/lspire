use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::menu::GameState;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSystemSet {
    Input,
    Physics,
    Camera,
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .configure_sets(Update, (
                GameSystemSet::Input,
                GameSystemSet::Physics,
                GameSystemSet::Camera,
            ).chain().run_if(in_state(GameState::InGame)));
    }
}
