use std::time::Instant;

use bevy::{
    asset::AssetServer,
    math::{Rect, Vec3},
    prelude::{
        default, Camera, Commands, DespawnRecursiveExt, Entity, Query, Res, ResMut, Transform, Visibility, With
    },
    sprite::{Sprite, SpriteBundle},
    time::Time,
};
use log::info;
use rand::Rng;

use crate::{
    config::Config, AdjustScale, AppState, TwitchMessage, UserAction, UserActionDetails, UserBundle, UserDetails, UserMarker
};

/// Spawn a new user entity in a random position
pub(crate) fn spawn_user(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    twitch_message: &TwitchMessage,
    config: &Config,
    rect: Rect,
) -> Entity {
    info!("New user: {}", twitch_message.user);
    let translation = Vec3::new(
        rand::thread_rng().gen_range((rect.max.x / -3.0)..(rect.max.x / 3.0)),
        -(rect.max.y / 2.0) + 25.0,
        0.0,
    );
    // If config.random_avatars is true, then look for a random file in ./assets/avatars using os
    // Otherwise, use the same avatar for all users
    let avatar_url = if config.random_avatars {
        let avatar_files = std::fs::read_dir("assets/avatars").unwrap();
        let avatar_files: Vec<String> = avatar_files
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        let random_avatar = rand::thread_rng().gen_range(0..avatar_files.len());
        format!("avatars/{}", avatar_files[random_avatar])
    } else {
        config.avatar_url.clone()
    };
    commands
        .spawn(UserBundle {
            marker: UserMarker {},
            details: UserDetails {
                _name: twitch_message.user.clone(),
            },
            sprite: SpriteBundle {
                texture: asset_server.load(&avatar_url),
                transform: Transform::from_translation(translation),
                visibility: Visibility::Hidden,
                ..default()
            },
            last_action: UserActionDetails {
                last_action: UserAction::Stop,
                time: Instant::now(),
            },
        }).insert(AdjustScale{})
        .id()
}

// Move avatars left and right randomly
pub(crate) fn move_users(
    mut user_query: Query<(&mut Transform, &mut Sprite, &mut UserActionDetails), With<UserMarker>>,
    camera_query: Query<&Camera>,
    time: Res<Time>,
    config: Res<Config>,
) {
    let mut rng = rand::thread_rng();
    let rect = camera_query.single().logical_viewport_rect().unwrap();
    for (mut transform, mut sprite, mut action) in user_query.iter_mut() {
        let now = Instant::now();
        let delta = time.delta_seconds();

        let wait_duration = match action.last_action {
            UserAction::Stop => config.wait_duration,
            _ => config.action_duration,
        };
        // Check if it's time to change the action
        if now.duration_since(action.time) > wait_duration {
            // Check if the user is close to the left edge
            let close_to_left_edge = transform.translation.x <= (rect.max.x / -2.0) + config.edge_buffer;
            // Check if the user is close to the right edge
            let close_to_right_edge = transform.translation.x >= (rect.max.x / 2.0) - config.edge_buffer;

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
                transform.translation.x -= config.avatar_move_speed * delta;
                sprite.flip_x = true;
            }
            UserAction::MoveRight => {
                transform.translation.x += config.avatar_move_speed * delta;
                sprite.flip_x = false;
            }
            UserAction::Stop => {}
            UserAction::_Bark => {}
        }
    }
}

// Check if avatars need to despawn due to inactivity
pub(crate) fn despawn_users(mut commands: Commands, mut app_state: ResMut<AppState>, config: Res<Config>) {
    let now = Instant::now();
    app_state.active_users.retain(|user_name, user| {
        if now.duration_since(user.last_message_time) > config.user_despawn_time {
            info!("Despawning user: {}", user_name);
            commands.entity(user.entity).despawn_recursive();
            false
        } else {
            true
        }
    });
}