use std::time::Instant;

use bevy::{
    asset::{AssetServer, Handle}, color::{Alpha, Color}, log::warn, math::{Vec2, Vec3}, prelude::{
        default, BuildChildren, Commands, DespawnRecursiveExt, Entity, Image, Query, Res, ResMut,
        Transform,
    }, render::texture::{ImageFormatSetting, ImageLoaderSettings}, sprite::{Anchor, Sprite, SpriteBundle}, text::{
        BreakLineOn, Font, JustifyText, Text, Text2dBounds, Text2dBundle, TextSection, TextStyle,
    }
};
use log::{debug, info};
use vleue_kinetoscope::{AnimatedImage, AnimatedImageBundle};

use crate::{config::Config, EmoteStorage, MessageSpawnTime};

// System to display message above the avatar's head
pub(crate) fn display_message(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    emote_store: &mut ResMut<EmoteStorage>,
    config: &Res<Config>,
    entity: Entity,
    message: String,
) {
    info!("Displaying message: {}", message);

    // Font MUST be monospace or the emotes will not align correctly
    let font = asset_server.load(&config.font_url);

    // Configure the message box
    let mut box_size = Vec2::new(config.message_box_width, 50.0);
    let mut box_position = Vec2::new(box_size.x * -0.5, config.message_box_vertical_offset);

    // debug!("Font size: {}", FONT_SIZE);
    // debug!("Font height: {}", font_height);
    // debug!("Font width: {}", font_width);

    let (text_sections, mut anim_emote_bundles, mut static_emote_bundles, lines, entries) =
        create_message_sections(asset_server, message, emote_store, font, config);

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

    box_size.y = (lines + 1.0) * (config.font_height() + config.line_space()) + config.top_margin() + 10.0;
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
    config: &Config,
) -> Transform {
    Transform::from_translation(Vec3::new(
        line_length - (config.font_width() * spacing_width / 2.0),
        -line_number * (config.font_height() + config.line_space()) - config.top_margin() - 0.5 * config.font_height(),
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
    config: &Config,
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
        font_size: config.font_size,
        color: Color::WHITE,
    };

    let mut entries = 0;
    for word in message.split_whitespace() {
        entries += 1;
        if let Some(emote) = emote_store.all.get(word).cloned() {
            // Get the emote normalisation factor
            let emote_norm = config.font_height() * config.emote_size_multiplier / emote.height.unwrap_or(0) as f32;

            let spacing_width = (emote.width.unwrap_or(0) as f32 * emote_norm / config.font_width()).ceil();
            // info!("Emote: {} Scale: {}", emote.name, emote_norm);
            // info!("Width {:?} Height {:?}", emote.width, emote.height);
            // info!("Spacing width: {}", spacing_width);

            // Calculate space for the emote
            let spacing = &(" ".repeat(spacing_width as usize - 1) + " "); // Must be a non-breaking space (U+00A0)

            // Check if the emote fits on the current line
            if (line_length + (config.font_width() * (spacing_width - 1.0))) > config.message_box_width {
                text_sections.push(TextSection::new(line.clone(), text_style.clone()));
                debug!("Section: {:?}| Length: {}", line, line_length);
                line = "".to_string();
                line_length = 0.0;
                line_number += 1.0;
            }

            line += spacing;
            line_length += spacing_width * config.font_width();

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
                            config
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
                            config
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
            if (line_length + ((word.len() as f32 + 1.0) * config.font_width())) > config.message_box_width {
                text_sections.push(TextSection::new(line.clone(), text_style.clone()));
                debug!("Section: {:?} Length: {}", line, line_length);
                line = "".to_string();
                line_length = 0.0;
                line_number += 1.0;
            }
            line += &format!("{} ", word);
            line_length += (word.len() as f32 + 1.0) * config.font_width();
        }
    }

    if !line.is_empty() {
        text_sections.push(TextSection::new(line.clone(), text_style.clone()));
    };
    debug!("Section: {:?}| Length: {}", line, line_length);

    (
        text_sections,
        anim_emote_bundles,
        static_emote_bundles,
        line_number,
        entries,
    )
}

// System to handle despawning messages after a certain time
pub(crate) fn despawn_messages(mut commands: Commands, query: Query<(Entity, &MessageSpawnTime)>, config: Res<Config>) {
    let now = Instant::now();
    for (entity, spawn_time) in query.iter() {
        if now.duration_since(spawn_time.0) > config.message_despawn_time {
            commands.entity(entity).despawn_recursive();
        }
    }
}
