use bevy::prelude::States;

#[derive(States, Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
pub enum GameStates {
    #[default]
    AssetLoading,
    WorldLoading,
    InGame,
}
