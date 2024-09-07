use std::time::Instant;

use bevy::{
    prelude::{Bundle, Component, Entity, Resource},
    sprite::{Sprite, SpriteBundle},
    text::Text2dBundle,
    utils::HashMap,
};
use tokio::sync::mpsc;

/// Twitch message struct
pub(crate) struct TwitchMessage {
    pub(crate) user: String,
    pub(crate) message: String,
}

// Wrap the mpsc::Receiver in a struct and derive Resource
#[derive(Resource)]
pub(crate) struct TwitchReceiver {
    pub(crate) receiver: mpsc::Receiver<TwitchMessage>,
}

/// App State struct stored as a Resource
#[derive(Resource)]
pub(crate) struct AppState {
    // #[deref]
    pub(crate) active_users: HashMap<String, User>,
}

/// Struct to store User in App State
pub(crate) struct User {
    pub(crate) entity: Entity,
    pub(crate) name: String,
    pub(crate) last_message_time: Instant,
}
/// Marker component to identify user entities
#[derive(Component)]
pub(crate) struct UserMarker {}

/// Component to store the user's Twitch details
#[derive(Component)]
pub(crate) struct UserDetails {
    pub(crate) name: String,
}

/// Emum representing possible actions for a user
#[derive(Component)]
pub(crate) enum UserAction {
    MoveLeft,
    MoveRight,
    Stop,
    Bark,
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

// Bundle used to create a new message
#[derive(Bundle)]
pub(crate) struct MessageBundle {
    pub(crate) text: Text2dBundle,
    // pub(crate) background: Sprite,
    pub(crate) time: MessageSpawnTime,
}
