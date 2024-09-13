use bevy::{asset::Handle, prelude::Image, render::texture::ImageFormat};
use serde::{Deserialize, Serialize};
use vleue_kinetoscope::AnimatedImage;

pub(crate) struct EmoteMeta {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) format: bevy::render::texture::ImageFormat,
}

pub(crate) struct LoadedEmote {
    pub(crate) name: String,
    pub(crate) animated: bool,
    pub(crate) animated_image: Option<Handle<AnimatedImage>>,
    pub(crate) static_image: Option<Handle<Image>>,
}

#[derive(Debug, Clone)]
enum EmoteSource {
    SevenTV,
    Twitch,
}

#[derive(Debug, Clone)]
pub(crate) struct Emote {
    pub(crate) _id: String,
    pub(crate) name: String,
    pub(crate) animated: bool,
    pub(crate) emote_url: String,
    pub(crate) source: EmoteSource,
    pub(crate) format: Option<ImageFormat>,
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
}

impl Emote {
    pub(crate) fn add_animated(&self, handle: Handle<AnimatedImage>) -> LoadedEmote {
        LoadedEmote {
            name: self.name.clone(),
            animated: self.animated,
            animated_image: Some(handle),
            static_image: None,
        }
    }

    pub(crate) fn add_static(&self, handle: Handle<Image>) -> LoadedEmote {
        LoadedEmote {
            name: self.name.clone(),
            animated: self.animated,
            animated_image: None,
            static_image: Some(handle),
        }
    }
}

impl From<twitch_irc::message::Emote> for Emote {
    fn from(emote: twitch_irc::message::Emote) -> Self {
        Self {
            _id: emote.id.clone(),
            name: emote.code,
            animated: false,
            emote_url: format!(
                "https://static-cdn.jtvnw.net/emoticons/v2/{}/default/light/4.0",
                emote.id
            ),
            source: EmoteSource::Twitch,
            format: None,
            width: None,
            height: None,
        }
    }
}

impl From<RawSevenTVEmote> for Emote {
    fn from(raw_emote: RawSevenTVEmote) -> Self {
        let largest_width_file = raw_emote
            .host
            .files
            .iter()
            .filter(|file| file.name.ends_with(".webp"))
            .max_by_key(|file| file.width);
        if let Some(file) = largest_width_file {
            let url = format!("https:{}/{}", raw_emote.host.url, &file.name);
            Self {
                _id: raw_emote.id,
                name: raw_emote.name,
                animated: raw_emote.animated,
                emote_url: url,
                source: EmoteSource::SevenTV,
                format: Some(ImageFormat::WebP),
                width: Some(file.width),
                height: Some(file.height),
            }
        } else {
            // Use technical difficulties emote if no files are found
            Self {
                _id: raw_emote.id,
                name: raw_emote.name,
                animated: true,
                emote_url: String::from(
                    "https://cdn.7tv.app/emote/63384017cf7eb48c4e731a79/4x.webp",
                ),
                source: EmoteSource::SevenTV,
                format: Some(ImageFormat::WebP),
                width: Some(128),
                height: Some(128),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SevenTVResponse {
    pub emote_set: SevenTVEmoteSet,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SevenTVEmoteSet {
    pub emotes: Vec<SevenTVEmoteBundle>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SevenTVEmoteBundle {
    pub data: RawSevenTVEmote,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct RawSevenTVEmote {
    pub id: String,
    pub name: String,
    pub animated: bool,
    pub host: SevenTVEmoteHost,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct SevenTVEmoteHost {
    pub url: String,
    pub files: Vec<SevenTVEmoteFile>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct SevenTVEmoteFile {
    pub name: String,
    pub static_name: String,
    pub width: u32,
    pub height: u32,
}
