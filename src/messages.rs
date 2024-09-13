use std::time::Instant;

use bevy::{
    asset::{AssetServer, Handle},
    color::{Alpha, Color},
    math::{Vec2, Vec3},
    prelude::{
        default, BuildChildren, Commands, DespawnRecursiveExt, Entity, Image, Query, Res, ResMut,
        Transform,
    },
    render::texture::{ImageFormatSetting, ImageLoaderSettings},
    sprite::{Anchor, Sprite, SpriteBundle},
    text::{
        BreakLineOn, Font, JustifyText, Text, Text2dBounds, Text2dBundle, TextSection, TextStyle,
    },
};
use log::{debug, info};
use vleue_kinetoscope::{AnimatedImage, AnimatedImageBundle};

use crate::{EmoteStorage, MessageSpawnTime, MESSAGE_DESPAWN_TIME};

const FONT_SIZE: f32 = 20.0;
const EMOTE_SIZE_MULTIPLIER: f32 = 1.7; // Emotes scale to font height * this value
const MESSAGE_BOX_VERTICAL_OFFSET: f32 = 35.0;
const MESSAGE_BOX_WIDTH: f32 = 200.0;
const FONT_HEIGHT: f32 = FONT_SIZE * 0.7;
const FONT_WIDTH: f32 = FONT_HEIGHT * 0.70;
const TOP_MARGIN: f32 = FONT_HEIGHT * 0.15;
const LINE_SPACE: f32 = FONT_HEIGHT * 0.55;

// System to display message above the avatar's head
pub(crate) fn display_message(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    emote_store: &mut ResMut<EmoteStorage>,
    entity: Entity,
    message: String,
) {
    info!("Displaying message: {}", message);

    // Font MUST be monospace or the emotes will not align correctly
    let font = asset_server.load("fonts/Unifontexmono-DYWdE.ttf");

    // Configure the message box
    let mut box_size = Vec2::new(MESSAGE_BOX_WIDTH, 50.0);
    let mut box_position = Vec2::new(box_size.x * -0.5, MESSAGE_BOX_VERTICAL_OFFSET);

    // debug!("Font size: {}", FONT_SIZE);
    // debug!("Font height: {}", font_height);
    // debug!("Font width: {}", font_width);

    let (text_sections, mut anim_emote_bundles, mut static_emote_bundles, lines, entries) =
        create_message_sections(asset_server, message, emote_store, font);

    // If there is only one emote, display it large above the avatar
    if entries == 1 {
        if anim_emote_bundles.len() == 1 {
            let mut emote = anim_emote_bundles.pop().unwrap();
            emote.transform = Transform::from_translation(Vec3::new(0.0, 50.0, 3.0))
                .with_scale(Vec3::splat(0.45));
            commands.entity(entity).with_children(|parent| {
                parent.spawn(emote).insert(MessageSpawnTime(Instant::now()));
            });
            return;
        } else if static_emote_bundles.len() == 1 {
            let mut emote = static_emote_bundles.pop().unwrap();
            emote.transform = Transform::from_translation(Vec3::new(0.0, 50.0, 3.0))
                .with_scale(Vec3::splat(0.45));
            commands.entity(entity).with_children(|parent| {
                parent.spawn(emote).insert(MessageSpawnTime(Instant::now()));
            });
            return;
        }
    }

    box_size.y = (lines + 1.0) * (FONT_HEIGHT + LINE_SPACE) + TOP_MARGIN + 10.0;
    box_position.y += box_size.y;

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK.with_alpha(0.4),
                custom_size: Some(Vec2::new(box_size.x, box_size.y)),
                anchor: Anchor::TopLeft,
                ..default()
            },
            transform: Transform::from_translation(box_position.extend(0.0)),
            ..default()
        })
        .set_parent(entity)
        .insert(MessageSpawnTime(Instant::now()))
        .with_children(|builder| {
            builder.spawn(Text2dBundle {
                text: Text {
                    sections: text_sections,
                    justify: JustifyText::Left,
                    linebreak_behavior: BreakLineOn::WordBoundary,
                },
                text_anchor: Anchor::TopLeft,
                // Wrap text in the rectangle
                text_2d_bounds: Text2dBounds { size: box_size },
                // ensure the text is drawn on top of the box
                // transform: Transform::from_translation(box_position.extend(1.0)),
                ..default()
            });
            for emote_bundle in anim_emote_bundles {
                builder.spawn(emote_bundle);
            }
            for emote_bundle in static_emote_bundles {
                builder.spawn(emote_bundle);
            }
        });
}

/// Calculate the transform for an emote based on the current line and line length
fn calculate_emote_transform(
    line_length: f32,
    line_number: f32,
    spacing_width: f32,
    emote_norm: f32,
) -> Transform {
    Transform::from_translation(Vec3::new(
        line_length - (FONT_WIDTH * spacing_width / 2.0),
        -line_number * (FONT_HEIGHT + LINE_SPACE) - TOP_MARGIN - 0.5 * FONT_HEIGHT,
        3.0,
    ))
    .with_scale(Vec3::splat(emote_norm))
}

/// Create the message sections and emote bundles by calculating the line breaks and emote positions
fn create_message_sections(
    asset_server: &Res<AssetServer>,
    message: String,
    emote_store: &mut ResMut<EmoteStorage>,
    font: Handle<Font>,
) -> (
    Vec<TextSection>,
    Vec<AnimatedImageBundle>,
    Vec<SpriteBundle>,
    f32,
    i32,
) {
    let mut text_sections: Vec<TextSection> = vec![];
    let mut anim_emote_bundles: Vec<AnimatedImageBundle> = vec![];
    let mut static_emote_bundles: Vec<SpriteBundle> = vec![];

    let mut line: String = "".to_string();
    let mut line_length = 0.0;
    let mut line_number = 0.0;

    let text_style = TextStyle {
        font,
        font_size: FONT_SIZE,
        color: Color::WHITE,
    };

    let mut entries = 0;
    for word in message.split_whitespace() {
        entries += 1;
        if let Some(emote) = emote_store.all.get(word).cloned() {
            // Check if the emote fits on the current line
            if (line_length + (3.0 * FONT_WIDTH)) > MESSAGE_BOX_WIDTH {
                text_sections.push(TextSection::new(line.clone(), text_style.clone()));
                debug!("Line: {} Length: {}", line, line_length);
                line = "".to_string();
                line_length = 0.0;
                line_number += 1.0;
            }

            // Get the emote normalisation factor
            let emote_norm = FONT_HEIGHT * EMOTE_SIZE_MULTIPLIER / emote.height.unwrap_or(0) as f32;

            // Add a space for the emote
            let spacing_width = (emote.width.unwrap_or(0) as f32 * emote_norm / FONT_WIDTH).ceil();
            let spacing = &" ".repeat(spacing_width as usize); // Must be a non-breaking space (U+00A0)
            line += spacing;
            line_length += spacing_width * FONT_WIDTH;

            // println!("Emote: {} Scale: {}", emote.name, emote_norm);
            // println!("Width {:?} Height {:?}", emote.width, emote.height);
            // println!("Spacing width: {}", spacing_width);

            match emote.animated {
                true => {
                    let handle: Handle<AnimatedImage>;
                    if let Some(loaded_emote) = emote_store.loaded.get(&emote.name) {
                        handle = loaded_emote
                            .animated_image
                            .as_ref()
                            .expect("Loaded animated emote has handle")
                            .clone_weak();
                    } else {
                        handle = asset_server.load::<AnimatedImage>(&emote.emote_url);
                        emote_store
                            .loaded
                            .insert(emote.name.clone(), emote.add_animated(handle.clone()));
                    };
                    anim_emote_bundles.push(AnimatedImageBundle {
                        animated_image: handle,
                        transform: calculate_emote_transform(
                            line_length,
                            line_number,
                            spacing_width,
                            emote_norm,
                        ),
                        sprite: Sprite {
                            color: Color::WHITE,
                            ..default()
                        },
                        ..default()
                    });
                }
                false => {
                    let handle: Handle<Image>;
                    if let Some(loaded_emote) = emote_store.loaded.get(&emote.name) {
                        handle = loaded_emote
                            .static_image
                            .as_ref()
                            .expect("Loaded static emote has handle")
                            .clone_weak();
                    } else {
                        handle = asset_server.load_with_settings::<Image, ImageLoaderSettings>(
                            &emote.emote_url,
                            move |s: &mut ImageLoaderSettings| {
                                s.format = ImageFormatSetting::Format(
                                    emote.format.expect("Emote has format"),
                                )
                            },
                        );
                        emote_store
                            .loaded
                            .insert(emote.name.clone(), emote.add_static(handle.clone()));
                    };
                    static_emote_bundles.push(SpriteBundle {
                        texture: handle,
                        transform: calculate_emote_transform(
                            line_length,
                            line_number,
                            spacing_width,
                            emote_norm,
                        ),
                        sprite: Sprite {
                            color: Color::WHITE,
                            ..default()
                        },
                        ..default()
                    });
                }
            }
        } else {
            // Check if the word fits on the current line
            if (line_length + (word.len() as f32 * FONT_WIDTH)) > MESSAGE_BOX_WIDTH {
                text_sections.push(TextSection::new(line.clone(), text_style.clone()));
                debug!("Line: {} Length: {}", line, line_length);
                line = "".to_string();
                line_length = 0.0;
                line_number += 1.0;
            }
            line += &format!("{} ", word);
            line_length += (word.len() as f32 + 1.0) * FONT_WIDTH;
        }
    }

    if !line.is_empty() {
        text_sections.push(TextSection::new(line.clone(), text_style.clone()));
    };

    (
        text_sections,
        anim_emote_bundles,
        static_emote_bundles,
        line_number,
        entries,
    )
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
