pub mod emote_types;
use bevy::{render::texture::ImageFormat, utils::HashMap};
use log::{info, warn};

use crate::emotes::emote_types::{Emote, EmoteMeta, SevenTVResponse};

const SEVEN_TV_URL: &str = "https://7tv.io/v3/users/twitch/";

pub(crate) async fn get_seventv_emotes(channel_id: String) -> HashMap<String, Emote> {
    info!("Getting the 7TV channel emotes");
    let response = reqwest::get(format!("{}{}", SEVEN_TV_URL, channel_id)).await;
    if response.is_err() {
        panic!("Cannot get 7tv emotes");
    }

    let response: SevenTVResponse = response.unwrap().json::<SevenTVResponse>().await.unwrap();
    response
        .emote_set
        .emotes
        .iter()
        .map(|emote| (emote.data.name.clone(), Emote::from(emote.data.clone())))
        .collect()
}

async fn get_image_meta(url: &str) -> EmoteMeta {
    // Initialize an HTTP client
    let client = reqwest::Client::new();

    // Fetch the first 4096 bytes (this should be enough for most image formats)
    let response = client
        .get(url)
        .header(reqwest::header::RANGE, "bytes=0-8096")
        .send()
        .await
        .expect("Successful emote header request")
        .bytes();

    // Create a cursor to read the in-memory byte stream
    let cursor = std::io::Cursor::new(response);

    // Use the `image` crate to read the image dimensions from the cursor
    let bytes = cursor.into_inner().await.unwrap();
    let reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .expect("Guessed format");

    let image_format = reader.format().expect("Image format from metadata");

    // Decode the image header to get the dimensions
    let dimensions = match reader.into_dimensions() {
        Ok(dimensions) => dimensions,
        Err(err) => {
            warn!("Error reading image dimensions: {}", err);
            warn!("URL: {}", url);
            return EmoteMeta {
                width: 0,
                height: 0,
                format: ImageFormat::Png,
            };
        }
    };
    let format = ImageFormat::from_image_crate_format(image_format)
        .expect("Image format converted to Bevy format");

    // debug!("Image format: {:?}", image_format);
    // debug!("Width: {}, Height: {}", dimensions.0, dimensions.1);
    // debug!("Format: {:?}", format);

    EmoteMeta {
        width: dimensions.0,
        height: dimensions.1,
        format,
    }
}

pub(crate) async fn update_emote_meta(emote: &mut Emote) {
    let meta = get_image_meta(&emote.emote_url).await;
    emote.width = Some(meta.width);
    emote.height = Some(meta.height);
    emote.format = Some(meta.format);
    match meta.format {
        ImageFormat::Png => {
            emote.animated = false;
        }
        ImageFormat::Gif => {
            emote.animated = true;
        }
        ImageFormat::WebP => {
            emote.animated = true;
        }
        _ => {
            warn!("Unsupported image format: {:?}", emote.format);
            emote.animated = false;
        }
    }
}
