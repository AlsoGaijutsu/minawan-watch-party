use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SevenTVResponse {
    pub emote_set: SevenTVEmoteSet,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SevenTVEmoteSet {
    pub emotes: Vec<SevenTVEmoteBundle>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SevenTVEmoteBundle {
    pub data: RawSevenTVEmote,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawSevenTVEmote {
    pub id: String,
    pub name: String,
    pub host: SevenTVEmoteHost,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SevenTVEmoteHost {
    pub url: String,
    pub files: Vec<SevenTVEmoteFile>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SevenTVEmoteFile {
    pub name: String,
    pub static_name: String,
    pub width: i64,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct SevenTVEmote {
    pub id: String,
    pub name: String,
    pub emote_url: String,
}


impl From<RawSevenTVEmote> for SevenTVEmote {
    fn from(raw_emote: RawSevenTVEmote) -> Self {
        let largest_width_file = raw_emote.host.files.iter().filter(|file| file.name.ends_with(".webp")).max_by_key(|file| file.width);
        if let Some(file) = largest_width_file {
            let url = format!("https:{}/{}", raw_emote.host.url, &file.name.replace(".webp", ".gif"));
            Self {
                id: raw_emote.id,
                name: raw_emote.name,
                emote_url: url,
            }
        } else {
            // Use technical difficulties emote if no files are found
            Self {
                id: raw_emote.id,
                name: raw_emote.name,
                emote_url: String::from("https://cdn.7tv.app/emote/63384017cf7eb48c4e731a79/4x.webp"),
            }
        }
    }
}