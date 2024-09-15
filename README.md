# Minawan Watch Party

Chill with all the minawan while you watch cerber's streams! Watch everyone walk around with snippits of conversation.
Want to personalise yourself? Draw a custom minawan and submit it so everyone can see you walk around!
Want to chill in the discord? Choose a channel and see minawan chatting on your taskbar!

## Instructions

The program works out of the box with no configuration. Simply run the exe and use `win + shift + arrow` to move the overlay to the correct window.
See [Configuration] for details on how to use the .ini if you want to change monitored stream, font, etc

## Roadmap

- [x] Get minawan walking around above taskbar
- [x] Create 7tv style message bubbles with emotes inline with text
- [ ] React to common leaving messages, eg
    - Goodnight variants sends minawan to sleep, icon fades
    - Goodbye variants makes door appear which minawan leave through
- [ ] Support messages in discord
- [ ] Support uploading custom minawan

## Configuration

Configuration is done through editing config.ini. Most of the time you should only need to edit CHANNEL_NAME and CHANNEL_ID

### Variables

#### [Channel]
- CHANNEL_NAME = The name of the twitch channel to monitor
- CHANNEL_ID = The id of the twitch channel. This can be gotten from https://www.streamweasels.com/tools/convert-twitch-username-to-user-id/

#### [General]
- SCALE = Adjust the scale / size of everything. Lower to make everything smaller

#### [Avatars]
- AVATAR_URL = Either a local path to an image or a link to an image
- RANDOM_AVATARS = If set to `true` then a random image from `assets/avatars` will be selected each time a minawan joins. (For now should be the same res as `avatar.png` 61x46, the most random res)
- ACTION_DURATION_MILIS = Minimum time and avatar should walk for
- WAIT_DURATION_MILIS = Minimum an avatar should be still for
- AVATAR_MOVE_SPEED = How quickly an avatar should move
- USER_DESPAWN_TIME_SECS = How many seconds an avatar should remain on screen without any messages being sent
- EDGE_BUFFER = How close avatars can get to the edge of the screen before turning back

#### [Messages]
- FONT_URL = Either a local path to an image or a link to a font. This font must be unicode or have the No-Break Space character (U+00A0)
- FONT_SIZE = Font size
- EMOTE_SIZE_MULTIPLIER = How large inline emotes should be. ~1.7 for 7tv style experience
- MESSAGE_BOX_VERTICAL_OFFSET = How far above avatars message boxes are
- MESSAGE_BOX_WIDTH = How wide message boxes are
- MESSAGE_DESPAWN_TIME_MILIS = How many miliseconds messages will show before despawning
