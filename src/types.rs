use std::time::Instant;

use bevy::{
    ecs::query::QueryData, prelude::{Bundle, Component, Entity, Resource}, sprite::SpriteBundle, utils::HashMap
};
use tokio::sync::mpsc;

use crate::emotes::emote_types::{Emote, LoadedEmote};

/// Marker component to identify avatars that need their scale adjusted
#[derive(Component)]
pub(crate) struct AdjustScaleOnce {
    pub(crate) height: f32,
}

/// Twitch message struct
pub(crate) struct TwitchMessage {
    pub(crate) user: String,
    pub(crate) message: String,
    pub(crate) emotes: Vec<Emote>,
}

// Wrap the mpsc::Receiver in a struct and derive Resource
#[derive(Resource)]
pub(crate) struct TwitchReceiver {
    pub(crate) receiver: mpsc::Receiver<TwitchMessage>,
}

/// Struct to store all emotes that have not been loaded yet
#[derive(Resource)]
pub(crate) struct EmoteStorage {
    pub(crate) all: HashMap<String, Emote>,
    pub(crate) loaded: HashMap<String, LoadedEmote>,
}

/// App State struct stored as a Resource
#[derive(Resource)]
pub(crate) struct AppState {
    pub(crate) active_users: HashMap<String, User>,
    pub(crate) program_state: ProgramState,
}

#[derive(Resource, Debug)]
pub(crate) enum ProgramState {
    Loading,
    Running,
}

/// Struct to store User in App State
pub(crate) struct User {
    pub(crate) entity: Entity,
    pub(crate) _name: String,
    pub(crate) last_message_time: Instant,
}
/// Marker component to identify user entities
#[derive(Component)]
pub(crate) struct UserMarker {}

/// Component to store the user's Twitch details
#[derive(Component)]
pub(crate) struct UserDetails {
    pub(crate) _name: String,
}

/// Emum representing possible actions for a user
#[derive(Component)]
pub(crate) enum UserAction {
    MoveLeft,
    MoveRight,
    Stop,
    _Bark,
}

/// Bundle to store the user's last action and the time it was performed
#[derive(Component)]
pub(crate) struct UserActionDetails {
    pub(crate) last_action: UserAction,
    pub(crate) time: Instant,
}

/// Bundle used to easily create a new user entity
#[derive(Bundle)]
pub(crate) struct UserBundle {
    pub(crate) marker: UserMarker,
    pub(crate) details: UserDetails,
    pub(crate) sprite: SpriteBundle,
    pub(crate) last_action: UserActionDetails,
}

// MessageSpawnTime
#[derive(Component)]
pub(crate) struct MessageSpawnTime(pub(crate) Instant);

impl From<Instant> for MessageSpawnTime {
    fn from(time: Instant) -> Self {
        Self(time)
    }
}
