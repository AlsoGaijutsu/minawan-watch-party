use std::time::Instant;

use bevy::{
    asset::{AssetServer, Handle}, color::{Alpha, Color}, math::{Vec2, Vec3}, prelude::{
        default, BuildChildren, Commands, DespawnRecursiveExt, Entity, Image, Query, Res, Transform
    }, reflect::Reflect, sprite::{Anchor, Sprite, SpriteBundle}, text::{BreakLineOn, JustifyText, Text, Text2dBounds, Text2dBundle, TextSection, TextStyle}
};
use log::{ debug, info };
use vleue_kinetoscope::{AnimatedImage, AnimatedImageBundle};

use crate::{MessageSpawnTime, SevenTVEmotes, MESSAGE_DESPAWN_TIME};

const FONT_SIZE: f32 = 15.0;

// System to display message above the avatar's head
pub(crate) fn display_message(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    emote_rec: &Res<SevenTVEmotes>,
    entity: Entity,
    message: String
) {
    info!("Displaying message: {}", message);

    // Calculate character sizes for emote positioning
    // Relies on the font being monospaced
    let font = asset_server.load("fonts/Unifontexmono-DYWdE.ttf");
    let font_height = FONT_SIZE * 0.7;
    let font_width = font_height * 0.7;
    let top_margin = font_height * 0.1;
    let line_space = font_height * 0.43;

    // Configure the message box
    let box_position = Vec2::new(0.0, 50.0);
    let box_size = Vec2::new(150.0, 50.0);
    let text_start = Transform::from_translation((box_size * Vec2::new(-0.5, 0.5)).extend(1.0));

    let mut sections: Vec<TextSection> = vec![];
    let mut anim_emote_bundles: Vec<AnimatedImageBundle> = vec![];
    let mut static_emote_bundles: Vec<SpriteBundle> = vec![];

    let mut line: String = "".to_string();
    let mut line_length = 0.0;
    let mut line_number = 0.0;

    let text_style = TextStyle {
        font,
        font_size: FONT_SIZE,
        ..default()
    };

    debug!("Font size: {}", FONT_SIZE);
    debug!("Font height: {}", font_height);
    debug!("Font width: {}", font_width);

    let mut entries: usize = 0; // Count of words in the message More efficient than split_whitespace().count() for long messages

    for word in message.split_whitespace() {
        entries += 1;
        if let Some(emote) = emote_rec.emotes.get(word) {
            // Check if the emote fits on the current line
            if (line_length + (3.0 * font_width)) > box_size.x {
                sections.push(TextSection::new(line.clone(), text_style.clone()));
                debug!("Line: {} Length: {}", line, line_length);
                line = "".to_string();
                line_length = 0.0;
                line_number += 1.0;
            }
            // Add a space for the emote
            let spacing_width = (emote.width as f32 / 64.0).floor();
            let spacing = &" ".repeat(spacing_width as usize);
            line += spacing;
            line_length += spacing.len() as f32 * font_width;

            debug!("Emote: {}", word);
            debug!("Emote width: {}", emote.width);
            debug!("Spacing width: {}", spacing_width);
            debug!("Position: {}, {}", line_length - (font_width * spacing.len() as f32 / 2.0), -line_number * font_height);
            debug!("Line: {} Char: {}", line_number, line_length);

            match emote.animated {
                true => {
                    anim_emote_bundles.push(AnimatedImageBundle {
                        animated_image: asset_server.load::<AnimatedImage>(&emote.emote_url),
                        transform: Transform::from_translation(
                            text_start.translation + Vec3::new(
                                line_length - (font_width * spacing.len() as f32 / 2.0), 
                                -line_number * (font_height + line_space) - top_margin - 0.5 * font_height, 
                                3.0))
                                .with_scale(Vec3::splat(0.1)),
                        sprite: Sprite {
                            color: Color::WHITE.with_alpha(1.0),
                            ..default()
                        },
                        ..default()
                    });
                },
                false => {
                    static_emote_bundles.push(SpriteBundle {
                        texture: asset_server.load::<Image>(&emote.emote_url),
                        transform: Transform::from_translation(
                            text_start.translation + Vec3::new(
                                line_length - (font_width * spacing.len() as f32 / 2.0), 
                                -line_number * (font_height + line_space) - top_margin - 0.5 * font_height, 
                                3.0))
                                .with_scale(Vec3::splat(0.1)),
                            sprite: Sprite {
                                color: Color::WHITE.with_alpha(1.0),
                                ..default()
                            },
                        ..default()
                    });
                }
                
            }
        } else {
            // Check if the word fits on the current line
            if (line_length + (word.len() as f32 * font_width)) > box_size.x {
                sections.push(TextSection::new(line.clone(), text_style.clone()));
                debug!("Line: {} Length: {}", line, line_length);
                line = "".to_string();
                line_length = 0.0;
                line_number += 1.0;
            }
            line += &format!("{} ", word);
            line_length += (word.len() as f32 + 1.0) * font_width;
        }
    }
    if !line.is_empty() {
        sections.push(TextSection::new(line.clone(), text_style.clone()));
    }
    
    if entries == 1 {
        if anim_emote_bundles.len() == 1 {
            let mut emote = anim_emote_bundles.pop().unwrap();
            emote.transform = Transform::from_translation(Vec3::new(0.0, 50.0, 3.0))
                .with_scale(Vec3::splat(0.3));
            commands.entity(entity).with_children(|parent| {
                parent.spawn(emote).insert(MessageSpawnTime(Instant::now()));
            });
            return;
        } else if static_emote_bundles.len() == 1 {
            let mut emote = static_emote_bundles.pop().unwrap();
            emote.transform = Transform::from_translation(Vec3::new(0.0, 50.0, 3.0))
                .with_scale(Vec3::splat(0.3));
            commands.entity(entity).with_children(|parent| {
                parent.spawn(emote).insert(MessageSpawnTime(Instant::now()));
            });
            return;
        }
    }

    commands.entity(entity).with_children(|parent| {
        parent.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK.with_alpha(0.4),
                custom_size: Some(Vec2::new(box_size.x, box_size.y)),
                ..default()
            },
            transform: Transform::from_translation(box_position.extend(0.0)),
            ..default()
        })
        .insert(MessageSpawnTime(Instant::now()))
        .with_children(|builder| {
            builder.spawn(Text2dBundle {
                text: Text {
                    sections,
                    justify: JustifyText::Left,
                    linebreak_behavior: BreakLineOn::WordBoundary,
                },
                text_anchor: Anchor::TopLeft,
                // Wrap text in the rectangle
                text_2d_bounds: Text2dBounds { size: box_size },
                // ensure the text is drawn on top of the box
                transform: text_start,
                ..default()
            });
            for emote_bundle in anim_emote_bundles {
                builder.spawn(emote_bundle);
            }
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