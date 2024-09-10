use bevy::{
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    utils::HashMap,
    window::PresentMode,
};
use bevy_web_asset::WebAssetPlugin;
use env_logger::Env;
#[allow(unused_imports)]
use log::{debug, info};
use emotes::{get_seventv_emotes, get_twitch_emotes};
use vleue_kinetoscope::AnimatedImagePlugin;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use twitch_irc::{
    login::StaticLoginCredentials, ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

mod types;
use types::*;

mod users;
use users::{despawn_users, move_users, spawn_user};

mod messages;
use messages::{despawn_messages, display_message};

mod emotes;

const CHANNEL: &str = "ironmouse";
const CHANNEL_ID: &str = "175831187";
const ACTION_DURATION: Duration = Duration::from_millis(800);
const WAIT_DURATION: Duration = Duration::from_secs(2);
const AVATAR_MOVE_SPEED: f32 = 100.0; // pixels per second
const USER_DESPAWN_TIME: Duration = Duration::from_secs(1800); // 30 minutes in seconds
const MESSAGE_DESPAWN_TIME: Duration = Duration::from_secs(10);

#[tokio::main] // We use Tokio's runtime since `twitch-irc` requires it
async fn main() {
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    // Create a channel to communicate between Twitch client and Bevy
    let (tx, rx) = mpsc::channel::<TwitchMessage>(100);

    // Start Twitch IRC client in a separate async task
    tokio::spawn(async move {
        start_twitch_client(tx).await;
    });

    // Set up Wgpu settings
    let wgpu_settings = WgpuSettings {
        backends: Some(Backends::VULKAN),
        ..Default::default()
    };

    // Run Bevy application
    App::new()
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(TwitchReceiver { receiver: rx })
        .insert_resource(SevenTVEmotes { emotes: HashMap::new() })
        .insert_resource(AppState {
            active_users: HashMap::new(),
        })
        .add_plugins(WebAssetPlugin)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Transparent Window".to_string(),
                        transparent: true,
                        decorations: false,
                        present_mode: PresentMode::Mailbox,
                        window_level: bevy::window::WindowLevel::AlwaysOnTop,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(wgpu_settings),
                    synchronous_pipeline_compilation: false,
                }),
        )
        .add_plugins(AnimatedImagePlugin)
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_seventv_emotes)
        .add_systems(Startup, setup_twitch_emotes)
        .add_systems(Update, move_users)
        .add_systems(Update, despawn_users)
        .add_systems(Update, despawn_messages)
        .add_systems(Update, handle_twitch_messages)
        // .add_systems(Update, debug_position)
        // .add_systems(Update, debug_camera)
        .run();
}

// Set up the camera and window
fn setup(mut commands: Commands, mut windows: Query<&mut Window>) {
    commands.spawn(Camera2dBundle::default());
    let mut window: Mut<'_, Window> = windows.single_mut();
    window.resolution.set_scale_factor_override(Some(1.0));
    window.cursor.hit_test = false;
    window.set_maximized(true);
    // commands.spawn(vleue_kinetoscope::AnimatedImageBundle {
    //     animated_image: asset_server.load("https://cdn.7tv.app/emote/66bd095b0d8502f0629f69de/4x.webp"),
    //     ..default()
    // });
}

fn setup_seventv_emotes(mut emotes_rec: ResMut<SevenTVEmotes>) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let emotes = rt.block_on(async { 
        get_seventv_emotes(CHANNEL_ID.to_string()).await 
    });

    emotes_rec.emotes.extend(emotes);
}

fn setup_twitch_emotes(mut emotes_rec: ResMut<SevenTVEmotes>) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let emotes = rt.block_on(async { 
        get_twitch_emotes(CHANNEL_ID.to_string()).await 
    });

    emotes_rec.emotes.extend(emotes);
}

#[allow(dead_code)]
fn debug_position(query: Query<&GlobalTransform, With<UserMarker>>) {
    for transform in query.iter() {
        info!("Avatar position: {:?}", transform.translation());
    }
}

#[allow(dead_code)]
fn debug_camera(query: Query<&Camera>) {
    let rect = query.single().logical_viewport_rect().unwrap();
    info!("Camera rect: {:?}", rect);
}

async fn start_twitch_client(tx: mpsc::Sender<TwitchMessage>) {
    let config = ClientConfig::new_simple(StaticLoginCredentials::anonymous());

    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    client.join(CHANNEL.to_string()).unwrap();

    // Listen to incoming Twitch messages and send them to Bevy via the channel
    while let Some(message) = incoming_messages.recv().await {
        if let twitch_irc::message::ServerMessage::Privmsg(msg) = message {
            info!("{}: {}", msg.sender.name, msg.message_text);
            let twitch_message = TwitchMessage {
                user: msg.sender.name.clone(),
                message: msg.message_text.clone(),
            };
            tx.send(twitch_message).await.unwrap();
        }
    }
}

/// System to handle incoming Twitch messages
fn handle_twitch_messages(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    emote_rec: Res<SevenTVEmotes>,
    query: Query<&Camera>,
    mut app_state: ResMut<AppState>,
    mut twitch_receiver: ResMut<TwitchReceiver>,
) {
    while let Ok(twitch_message) = twitch_receiver.receiver.try_recv() {
        // Check if the user already exists
        if let Some(user) = app_state.active_users.get_mut(&twitch_message.user) {
            // Update the user's last message time and display the message
            display_message(
                &mut commands,
                &asset_server,
                &emote_rec,
                user.entity,
                twitch_message.message,
            );
            user.last_message_time = Instant::now();
        } else {
            // Add new user and spawn their avatar
            let rect = query.single().logical_viewport_rect().unwrap();
            let entity = spawn_user(&mut commands, &asset_server, &twitch_message, rect);
            app_state.active_users.insert(
                twitch_message.user.clone(),
                User {
                    entity,
                    _name: twitch_message.user.clone(),
                    last_message_time: Instant::now(),
                },
            );
            display_message(
                &mut commands,
                &asset_server,
                &emote_rec,
                entity,
                twitch_message.message,
            );
        }
    }
}
