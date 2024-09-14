use std::time::Duration;

use bevy::prelude::Resource;
use ini::Ini;

#[derive(Clone, Resource)]
pub(crate) struct Config {
    pub(crate) channel_name: String,
    pub(crate) channel_id: String,
    pub(crate) scale: f32,
    pub(crate) avatar_url: String,
    pub(crate) action_duration: Duration,
    pub(crate) wait_duration: Duration,
    pub(crate) avatar_move_speed: f32,
    pub(crate) user_despawn_time: Duration,
    pub(crate) edge_buffer: f32,
    pub(crate) font_url: String,
    pub(crate) font_size: f32,
    pub(crate) emote_size_multiplier: f32,
    pub(crate) message_box_vertical_offset: f32,
    pub(crate) message_box_width: f32,
    pub(crate) message_despawn_time: Duration,
}

impl Config {
    pub(crate) fn font_height(&self) -> f32 {
        self.font_size * 0.7
    }

    pub(crate) fn font_width(&self) -> f32 {
        self.font_height() * 0.67
    }

    pub(crate) fn top_margin(&self) -> f32 {
        self.font_height() * 0.15
    }

    pub(crate) fn line_space(&self) -> f32 {
        self.font_height() * 0.43
    }
}

pub(crate) fn load_config(filename: &str) -> Config {
    let conf = Ini::load_from_file(filename).expect("Failed to load config.ini");

    // Load [Channel] section
    let channel_section = conf
        .section(Some("Channel"))
        .expect("Missing [Channel] section");

    let channel_name = channel_section
        .get("CHANNEL_NAME")
        .expect("Missing CHANNEL_NAME")
        .to_string();

    let channel_id = channel_section
        .get("CHANNEL_ID")
        .expect("Missing CHANNEL_ID")
        .to_string();

    // Load [General] section
    let general_section = conf
        .section(Some("General"))
        .expect("Missing [General] section");

    let scale = general_section
        .get("SCALE")
        .expect("Missing SCALE")
        .parse::<f32>()
        .expect("Invalid SCALE");

    // Load [Avatars] section
    let avatars_section = conf
        .section(Some("Avatars"))
        .expect("Missing [Avatars] section");

    let avatar_url = avatars_section
        .get("AVATAR_URL")
        .expect("Missing AVATAR_URL")
        .to_string();

    let action_duration = Duration::from_millis(
        avatars_section
            .get("ACTION_DURATION_MILIS")
            .expect("Missing ACTION_DURATION_MILIS")
            .parse::<u64>()
            .expect("Invalid ACTION_DURATION_MILIS")
    );

    let wait_duration = Duration::from_millis(
        avatars_section
            .get("WAIT_DURATION_MILIS")
            .expect("Missing WAIT_DURATION_MILIS")
            .parse::<u64>()
            .expect("Invalid WAIT_DURATION_MILIS")
    );

    let avatar_move_speed = avatars_section
        .get("AVATAR_MOVE_SPEED")
        .expect("Missing AVATAR_MOVE_SPEED")
        .parse::<f32>()
        .expect("Invalid AVATAR_MOVE_SPEED");

    let user_despawn_time = Duration::from_secs(
        avatars_section
            .get("USER_DESPAWN_TIME_SECS")
            .expect("Missing USER_DESPAWN_TIME_SECS")
            .parse::<u64>()
            .expect("Invalid USER_DESPAWN_TIME_SECS")
    );

    let edge_buffer = avatars_section
        .get("EDGE_BUFFER")
        .expect("Missing EDGE_BUFFER")
        .parse::<f32>()
        .expect("Invalid EDGE_BUFFER");

    // Load [Messages] section
    let general_section = conf.section(Some("Messages")).expect("Missing [General] section");

    let font_url = general_section
        .get("FONT_URL")
        .expect("Missing FONT_URL")
        .to_string();

    let font_size = general_section
        .get("FONT_SIZE")
        .expect("Missing FONT_SIZE")
        .parse::<f32>()
        .expect("Invalid FONT_SIZE");

    let emote_size_multiplier = general_section
        .get("EMOTE_SIZE_MULTIPLIER")
        .expect("Missing EMOTE_SIZE_MULTIPLIER")
        .parse::<f32>()
        .expect("Invalid EMOTE_SIZE_MULTIPLIER");

    let message_box_vertical_offset = general_section
        .get("MESSAGE_BOX_VERTICAL_OFFSET")
        .expect("Missing MESSAGE_BOX_VERTICAL_OFFSET")
        .parse::<f32>()
        .expect("Invalid MESSAGE_BOX_VERTICAL_OFFSET");

    let message_box_width = general_section
        .get("MESSAGE_BOX_WIDTH")
        .expect("Missing MESSAGE_BOX_WIDTH")
        .parse::<f32>()
        .expect("Invalid MESSAGE_BOX_WIDTH");

    let message_despawn_time = Duration::from_millis(
        general_section
            .get("MESSAGE_DESPAWN_TIME_MILIS")
            .expect("Missing MESSAGE_DESPAWN_TIME_MILIS")
            .parse::<u64>()
            .expect("Invalid MESSAGE_DESPAWN_TIME_MILIS")
    );

    Config {
        channel_name,
        channel_id,
        scale,
        avatar_url,
        action_duration,
        wait_duration,
        avatar_move_speed,
        user_despawn_time,
        edge_buffer,
        font_url,
        font_size,
        emote_size_multiplier,
        message_box_vertical_offset,
        message_box_width,
        message_despawn_time,
    }
}
