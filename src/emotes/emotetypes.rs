use serde::{Deserialize, Serialize};

pub(crate) struct Emote {
    pub(crate) _id: String,
    pub(crate) name: String,
    pub(crate) animated: bool,
    pub(crate) emote_url: String,
    pub(crate) width: i64,
    pub(crate) height: i64,
}

impl From<RawSevenTVEmote> for Emote {
    fn from(raw_emote: RawSevenTVEmote) -> Self {
        let largest_width_file = raw_emote.host.files.iter().filter(|file| file.name.ends_with(".webp")).max_by_key(|file| file.width);
        if let Some(file) = largest_width_file {
            let url = format!("https:{}/{}", raw_emote.host.url, &file.name);
            Self {
                _id: raw_emote.id,
                name: raw_emote.name,
                animated: raw_emote.animated,
                emote_url: url,
                width: file.width,
                height: file.height,
            }
        } else {
            // Use technical difficulties emote if no files are found
            Self {
                _id: raw_emote.id,
                name: raw_emote.name,
                animated: true,
                emote_url: String::from("https://cdn.7tv.app/emote/63384017cf7eb48c4e731a79/4x.webp"),
                width: 128,
                height: 128,
            }
        }
    }
}

impl From<TwitchEmote> for Emote {
    fn from(twitch_emote: TwitchEmote) -> Self {
        Self {
            _id: twitch_emote.id,
            name: twitch_emote.name,
            animated: twitch_emote.format.contains(&String::from("animated")),
            emote_url: twitch_emote.images.url_4x.clone(),
            width: 128,
            height: 128,
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
    pub width: i64,
    pub height: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TwitchEmoteResponse {
    pub(crate) emotes: Vec<TwitchEmote>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TwitchEmote {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) images: TwitchEmoteImages,
    pub(crate) format: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TwitchEmoteImages {
    pub(crate) url_1x: String,
    pub(crate) url_2x: String,
    pub(crate) url_4x: String,
}