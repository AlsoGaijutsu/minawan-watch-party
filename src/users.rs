use std::time::Instant;

use bevy::{
    asset::AssetServer,
    color::Color,
    math::{Rect, Vec2, Vec3},
    prelude::{
        default, BuildChildren, Camera, Commands, DespawnRecursiveExt, Entity, Query, Res, ResMut,
        Transform, With,
    },
    sprite::{Anchor, Sprite, SpriteBundle},
    text::{JustifyText, Text, Text2dBounds, Text2dBundle, TextStyle},
    time::Time,
};
use log::info;
use rand::Rng;

use crate::{
    AppState, MessageBundle, MessageSpawnTime, TwitchMessage, UserAction, UserActionDetails,
    UserBundle, UserDetails, UserMarker, ACTION_DURATION, AVATAR_MOVE_SPEED, MESSAGE_DESPAWN_TIME,
    USER_DESPAWN_TIME, WAIT_DURATION,
};

/// Spawn a new user entity in a random position
pub(crate) fn spawn_user(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    twitch_message: &TwitchMessage,
    rect: Rect,
) -> Entity {
    info!("New user: {}", twitch_message.user);
    let translation = Vec3::new(
        rand::thread_rng().gen_range((rect.max.x / -3.0)..(rect.max.x / 3.0)),
        -(rect.max.y / 2.0) + 50.0,
        0.0,
    );
    commands
        .spawn(UserBundle {
            marker: UserMarker {},
            details: UserDetails {
                name: twitch_message.user.clone(),
            },
            sprite: SpriteBundle {
                texture: asset_server.load("https://cdn.7tv.app/emote/66bd095b0d8502f0629f69de/4x.webp"),
                transform: Transform {
                    translation,
                    scale: Vec3::new(0.5, 0.5, 0.5),
                    ..default()
                },
                ..default()
            },
            last_action: UserActionDetails {
                last_action: UserAction::Stop,
                time: Instant::now(),
            },
        })
        .id()
}

// Move avatars left and right randomly
pub(crate) fn move_users(
    mut user_query: Query<(&mut Transform, &mut UserActionDetails), With<UserMarker>>,
    camera_query: Query<&Camera>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();
    let rect = camera_query.single().logical_viewport_rect().unwrap();
    for (mut transform, mut action) in user_query.iter_mut() {
        let now = Instant::now();
        let delta = time.delta_seconds();

        let wait_duration = match action.last_action {
            UserAction::Stop => WAIT_DURATION,
            _ => ACTION_DURATION,
        };
        // Check if it's time to change the action
        if now.duration_since(action.time) > wait_duration {
            // Check if the user is close to the left edge
            let close_to_left_edge = transform.translation.x <= (rect.max.x / -2.0) + 100.0;
            // Check if the user is close to the right edge
            let close_to_right_edge = transform.translation.x >= (rect.max.x / 2.0) - 100.0;

            action.last_action = match rng.gen_range(0..3) {
                0 if close_to_left_edge => UserAction::MoveRight,
                1 if close_to_right_edge => UserAction::MoveLeft,
                0 => UserAction::MoveLeft,
                1 => UserAction::MoveRight,
                _ => UserAction::Stop,
            };
            action.time = now;
        }

        // Perform the action
        match action.last_action {
            UserAction::MoveLeft => {
                transform.translation.x -= AVATAR_MOVE_SPEED * delta;
            }
            UserAction::MoveRight => {
                transform.translation.x += AVATAR_MOVE_SPEED * delta;
            }
            UserAction::Stop => {}
            UserAction::Bark => {}
        }
    }
}

// Check if avatars need to despawn due to inactivity
pub(crate) fn despawn_users(mut commands: Commands, mut app_state: ResMut<AppState>) {
    let now = Instant::now();
    app_state.active_users.retain(|user_name, user| {
        if now.duration_since(user.last_message_time) > USER_DESPAWN_TIME {
            info!("Despawning user: {}", user_name);
            commands.entity(user.entity).despawn_recursive();
            false
        } else {
            true
        }
    });
}
