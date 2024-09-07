use std::time::Instant;

use bevy::{
    asset::AssetServer,
    color::Color,
    math::{Vec2, Vec3},
    prelude::{
        default, BuildChildren, Commands, DespawnRecursiveExt, Entity, Query, Res, Transform,
    },
    sprite::{Sprite, SpriteBundle},
    text::{JustifyText, Text, Text2dBounds, Text2dBundle, TextStyle},
};
use log::info;

use crate::{MessageBundle, MessageSpawnTime, MESSAGE_DESPAWN_TIME};

// System to display message above the avatar's head
pub(crate) fn display_message(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    entity: Entity,
    message: &str,
) {
    info!("Displaying message: {}", message);
    // let font_handle = asset_server.load("fonts/Comic-Sans-MS.ttf");

    // Parameters for the message and background
    let font_size = 20.0;
    let padding = 10.0;
    let max_width = 250.0; // Maximum width for the text box

    // Estimate the height of the text box based on the number of lines
    let text_length = message.len() as f32;
    let max_chars_per_line = max_width / (font_size * 0.6); // Estimate characters per line based on font size
    let num_lines = (text_length / max_chars_per_line).ceil(); // Number of lines after wrapping
    let text_height = num_lines * font_size * 4.0;

    let background_size = Vec2::new(max_width + 2.0 * padding, text_height + 2.0 * padding);
    let background_color = Color::linear_rgba(1.0, 1.0, 1.0, 1.0); // Semi-transparent black background
    let transform = Transform::from_translation(Vec3::new(0.0, 150.0, 1.0));
    commands.entity(entity).with_children(|parent| {
        parent
            .spawn(MessageBundle {
                text: Text2dBundle {
                    text: Text::from_section(
                        message,
                        TextStyle {
                            font_size: 40.0,
                            ..default()
                        },
                    )
                    .with_justify(JustifyText::Left),
                    transform,
                    text_2d_bounds: Text2dBounds {
                        size: Vec2 {
                            x: max_width,
                            y: text_height,
                        },
                    },
                    ..default()
                },
                time: MessageSpawnTime::from(Instant::now()),
            })
            .insert(Sprite {
                color: background_color,
                custom_size: Some(background_size),
                ..default()
            });
    });
}

// System to handle despawning messages after a certain time
pub(crate) fn despawn_messages(mut commands: Commands, query: Query<(Entity, &MessageSpawnTime)>) {
    let now = Instant::now();
    for (entity, spawn_time) in query.iter() {
        if now.duration_since(spawn_time.0) > MESSAGE_DESPAWN_TIME {
            commands.entity(entity).despawn_recursive();
        }
    }
}