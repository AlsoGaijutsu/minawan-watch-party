use std::time::Instant;

use bevy::{
    asset::AssetServer,
    math::{Rect, Vec3},
    prelude::{
        default, Camera, Commands, DespawnRecursiveExt, Entity, Query, Res, ResMut, Transform, With,
    },
    sprite::{Sprite, SpriteBundle},
    time::Time,
};
use log::info;
use rand::Rng;

use crate::{
    AppState, TwitchMessage, UserAction, UserActionDetails, UserBundle, UserDetails, UserMarker,
    ACTION_DURATION, AVATAR_MOVE_SPEED, USER_DESPAWN_TIME, WAIT_DURATION,
};

const EDGE_BUFFER: f32 = 20.0; // Buffer to prevent avatars from going off screen

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
        -(rect.max.y / 2.0) + 25.0,
        0.0,
    );
    commands
        .spawn(UserBundle {
            marker: UserMarker {},
            details: UserDetails {
                _name: twitch_message.user.clone(),
            },
            sprite: SpriteBundle {
                texture: asset_server.load("images/avatar.png"),
                transform: Transform::from_translation(translation),
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
    mut user_query: Query<(&mut Transform, &mut Sprite, &mut UserActionDetails), With<UserMarker>>,
    camera_query: Query<&Camera>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();
    let rect = camera_query.single().logical_viewport_rect().unwrap();
    for (mut transform, mut sprite, mut action) in user_query.iter_mut() {
        let now = Instant::now();
        let delta = time.delta_seconds();

        let wait_duration = match action.last_action {
            UserAction::Stop => WAIT_DURATION,
            _ => ACTION_DURATION,
        };
        // Check if it's time to change the action
        if now.duration_since(action.time) > wait_duration {
            // Check if the user is close to the left edge
            let close_to_left_edge = transform.translation.x <= (rect.max.x / -2.0) + EDGE_BUFFER;
            // Check if the user is close to the right edge
            let close_to_right_edge = transform.translation.x >= (rect.max.x / 2.0) - EDGE_BUFFER;

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
                sprite.flip_x = true;
            }
            UserAction::MoveRight => {
                transform.translation.x += AVATAR_MOVE_SPEED * delta;
                sprite.flip_x = false;
            }
            UserAction::Stop => {}
            UserAction::_Bark => {}
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
