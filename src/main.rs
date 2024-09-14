#![windows_subsystem = "windows"]
use bevy::{
    // log::LogPlugin,
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    utils::HashMap,
    window::PresentMode,
};
use bevy_web_asset::WebAssetPlugin;
use emotes::{get_seventv_emotes, update_emote_meta};
use log::info;
use std::time::{Duration, Instant};
use tokio::{sync::mpsc, time::sleep};
use twitch_irc::{
    login::StaticLoginCredentials, ClientConfig, SecureTCPTransport, TwitchIRCClient,
};
use vleue_kinetoscope::AnimatedImagePlugin;
use env_logger::Env;

mod types;
use types::*;

mod users;
use users::{despawn_users, move_users, spawn_user};

mod messages;
use messages::{despawn_messages, display_message};

mod emotes;

const CHANNEL: &str = "cerbervt";
const CHANNEL_ID: &str = "852880224";
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
        .insert_resource(EmoteStorage {
            all: HashMap::new(),
            loaded: HashMap::new(),
        })
        .insert_resource(AppState {
            active_users: HashMap::new(),
            program_state: ProgramState::Loading,
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
                })
                // .disable::<LogPlugin>(),
        )
        .add_plugins(AnimatedImagePlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_users,
                despawn_users,
                despawn_messages,
                handle_twitch_messages,
            ),
        )
        .run();
}

// Set up the camera and window
fn setup(
    mut commands: Commands,
    mut windows: Query<&mut Window>,
    mut emotes_rec: ResMut<EmoteStorage>,
    mut app_state: ResMut<AppState>,
) {
    commands.spawn(Camera2dBundle::default());
    let mut window: Mut<'_, Window> = windows.single_mut();
    window.resolution.set_scale_factor_override(Some(1.0));
    window.cursor.hit_test = false;
    window.set_maximized(true);

    setup_seventv_emotes(&mut emotes_rec);

    app_state.program_state = ProgramState::Running;
}

fn setup_seventv_emotes(emotes_rec: &mut ResMut<EmoteStorage>) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let emotes = rt.block_on(async { get_seventv_emotes(CHANNEL_ID.to_string()).await });

    emotes_rec.all.extend(emotes);
}

async fn start_twitch_client(tx: mpsc::Sender<TwitchMessage>) {
    let config = ClientConfig::new_simple(StaticLoginCredentials::anonymous());

    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    client.join(CHANNEL.to_string()).unwrap();

    sleep(Duration::from_millis(2000)).await;

    let mut seen_emotes: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Listen to incoming Twitch messages and send them to Bevy via the channel
    while let Some(message) = incoming_messages.recv().await {
        if let twitch_irc::message::ServerMessage::Privmsg(msg) = message {
            info!("{}: {}", msg.sender.name, msg.message_text);
            let mut twitch_message = TwitchMessage {
                user: msg.sender.name.clone(),
                message: msg.message_text.clone(),
                emotes: msg.emotes.into_iter().map(|emote| emote.into()).collect(),
            };

            let mut new_emotes: std::collections::HashSet<String> =
                std::collections::HashSet::new();

            for emote in twitch_message
                .emotes
                .iter_mut()
                .filter(|emote| !seen_emotes.contains(&emote.name))
            {
                update_emote_meta(emote).await;
                new_emotes.insert(emote.name.clone());
            }
            seen_emotes.extend(new_emotes);
            tx.send(twitch_message).await.unwrap(); // Use the cloned tx value
        }
    }
}

/// System to handle incoming Twitch messages
fn handle_twitch_messages(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut emote_rec: ResMut<EmoteStorage>,
    query: Query<&Camera>,
    mut app_state: ResMut<AppState>,
    mut twitch_receiver: ResMut<TwitchReceiver>,
) {
    while let Ok(twitch_message) = twitch_receiver.receiver.try_recv() {
        // Add any new emotes to the storage
        for emote in twitch_message.emotes.iter() {
            emote_rec
                .all
                .entry(emote.name.clone())
                .or_insert(emote.clone());
        }
        // Check if the user already exists
        if let Some(user) = app_state.active_users.get_mut(&twitch_message.user) {
            // Update the user's last message time and display the message
            display_message(
                &mut commands,
                &asset_server,
                &mut emote_rec,
                user.entity,
                twitch_message.message,
            );
            // user.last_message = Some(message);
            user.last_message_time = Instant::now();
        } else {
            // Add new user and spawn their avatar
            let rect = query.single().logical_viewport_rect().unwrap();
            let entity = spawn_user(&mut commands, &asset_server, &twitch_message, rect);
            display_message(
                &mut commands,
                &asset_server,
                &mut emote_rec,
                entity,
                twitch_message.message,
            );
            app_state.active_users.insert(
                twitch_message.user.clone(),
                User {
                    entity,
                    _name: twitch_message.user.clone(),
                    last_message_time: Instant::now(),
                },
            );
        }
    }
}
