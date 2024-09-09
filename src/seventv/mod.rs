pub mod seventvtypes;
use bevy::utils::HashMap;
use log::{ info, warn };

use crate::seventv::seventvtypes::{SevenTVEmote, SevenTVResponse};

const SEVEN_TV_URL: &str = "https://7tv.io/v3/users/twitch/85498365";

pub(crate) async fn get_seventv_emotes(channel_id: String) -> HashMap<String, String> {
    info!("Getting the 7TV channel emotes");
    let response = reqwest::get(SEVEN_TV_URL).await;
    if response.is_err() {
        warn!("Cannot get 7tv emotes");
        return HashMap::new();
    }

    let response: SevenTVResponse = response.unwrap().json::<SevenTVResponse>().await.unwrap();
    let seventv_emotes: Vec<SevenTVEmote> = response.emote_set
        .emotes
        .iter()
        .map(|emote| SevenTVEmote::from(emote.data.clone()))
        .collect();

    let emotes: HashMap<String, String> = seventv_emotes
        .into_iter()
        .map(|emote| (emote.name.clone(), emote.emote_url))
        .collect();

    emotes
}