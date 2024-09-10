pub mod emotetypes;
use bevy::utils::HashMap;
use log::{ info, warn };

use crate::emotes::emotetypes::{Emote, SevenTVResponse, TwitchEmoteResponse};

const SEVEN_TV_URL: &str = "https://7tv.io/v3/users/twitch/";
const TWITCH_URL: &str = "https://twitch-middleman.clueless.workers.dev/";

pub(crate) async fn get_seventv_emotes(channel_id: String) -> HashMap<String, Emote> {
    info!("Getting the 7TV channel emotes");
    let response = reqwest::get(format!("{}{}", SEVEN_TV_URL, channel_id)).await;
    if response.is_err() {
        warn!("Cannot get 7tv emotes");
        return HashMap::new();
    }

    let response: SevenTVResponse = response.unwrap().json::<SevenTVResponse>().await.unwrap();
    response.emote_set
        .emotes
        .iter()
        .map(|emote| (emote.data.name.clone(), Emote::from(emote.data.clone())))
        .collect()
}

pub(crate) async fn get_twitch_emotes(channel_id: String) -> HashMap<String, Emote> {
    info!("Getting the Twitch channel emotes");
    let response = reqwest::get(format!("{}{}", TWITCH_URL, channel_id)).await;
    if response.is_err() {
        warn!("Cannot get Twitch emotes");
        return HashMap::new();
    }

    let response: TwitchEmoteResponse = response.unwrap().json::<TwitchEmoteResponse>().await.unwrap();

    response.emotes
        .into_iter()
        .map(|emote| (emote.name.clone(), Emote::from(emote)))
        .collect()
}