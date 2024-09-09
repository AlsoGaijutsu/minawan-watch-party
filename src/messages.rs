use std::time::Instant;

use bevy::{
    asset::{AssetServer, Assets},
    color::{Alpha, Color},
    math::{Vec2, Vec3},
    prelude::{
        default, BuildChildren, Commands, DespawnRecursiveExt, Entity, NodeBundle, Query, Res, Transform, TransformBundle
    },
    text::{Text, Text2dBundle, TextStyle}, ui::{AlignItems, BackgroundColor, FlexDirection, Style, Val}, utils::HashMap,
};
use log::info;
use vleue_kinetoscope::AnimatedImageBundle;

use crate::{MessageSpawnTime, SevenTVEmotes, MESSAGE_DESPAWN_TIME};

// System to display message above the avatar's head
pub(crate) fn display_message(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    emote_rec: &Res<SevenTVEmotes>,
    entity: Entity,
    message: &str
) {
    info!("Displaying message: {}", message);
    // Create a root node for the text and emotes
    let root_entity = commands.entity(entity).with_children(|parent| {
        parent.spawn(TransformBundle {
            local: Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
            ..default()
        });
    }).id();

    // Split the message by spaces or other delimiters to extract text and emotes
    let parts = message.split_whitespace();
    let mut cont_string = "".to_string();

    // Iterate through each part of the message
    for (i, part) in parts.enumerate() {
        if i >= 10 {
            break;
        }
        if let Some(emote_url) = emote_rec.emotes.get(part) {
            // Create a TextBundle from the text so far
            let cloned_cont_string = cont_string.clone(); // Clone the cont_string variable
            commands.entity(root_entity).with_children(|parent| {
                parent.spawn(Text2dBundle {
                    text: Text::from_section(
                        cloned_cont_string, // Use the cloned_cont_string variable
                        TextStyle {
                            font: asset_server.load("fonts/Comic-Sans-MS.ttf"), // Ensure you have a font asset
                            font_size: 20.0,
                            color: Color::WHITE,
                        }
                    ),
                    ..default()
                });
            });
            cont_string.clear();
            // If part is an emote, create an ImageBundle
            info!("Displaying emote: {}", part);
            info!("Emote URL: {}", emote_url);
            commands.entity(root_entity).with_children(|parent| {
                parent.spawn(AnimatedImageBundle {
                    animated_image: asset_server.load(emote_url), // Load the image asset
                    transform: Transform::from_scale(Vec3::new(0.2, 0.2, 0.2)), // Scale down the image
                    ..default()
                });
            });
        } else {
            cont_string.push(' ');
            cont_string.push_str(part);
        }
    }
    commands.entity(root_entity).with_children(|parent| {
        parent.spawn(Text2dBundle {
            text: Text::from_section(
                cont_string, // Use the cloned_cont_string variable
                TextStyle {
                    font: asset_server.load("fonts/Comic-Sans-MS.ttf"), // Ensure you have a font asset
                    font_size: 20.0,
                    color: Color::WHITE,
                }
            ),
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