use rodio::{Decoder, OutputStream, Source};
use shakmaty::{Chess, Move, Position};
use std::io::Cursor;
use std::sync::mpsc::{channel, Sender};
use std::sync::OnceLock;

const SOUND_MOVE_SELF: &[u8] = include_bytes!("../assets/sounds/move-self.mp3");
const SOUND_MOVE_OPPONENT: &[u8] = include_bytes!("../assets/sounds/move-opponent.mp3");
const SOUND_CAPTURE: &[u8] = include_bytes!("../assets/sounds/capture.mp3");
const SOUND_CHECK: &[u8] = include_bytes!("../assets/sounds/move-check.mp3");
const SOUND_CASTLE: &[u8] = include_bytes!("../assets/sounds/castle.mp3");
const SOUND_PROMOTE: &[u8] = include_bytes!("../assets/sounds/promote.mp3");
const SOUND_GAME_START: &[u8] = include_bytes!("../assets/sounds/game-start.mp3");
const SOUND_GAME_END: &[u8] = include_bytes!("../assets/sounds/game-end.mp3");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundType {
    Move,
    MoveOpponent,
    Capture,
    Check,
    Castle,
    Promote,
    GameStart,
    GameEnd,
}

impl SoundType {
    pub fn bytes(&self) -> &'static [u8] {
        match self {
            SoundType::Move => SOUND_MOVE_SELF,
            SoundType::MoveOpponent => SOUND_MOVE_OPPONENT,
            SoundType::Capture => SOUND_CAPTURE,
            SoundType::Check => SOUND_CHECK,
            SoundType::Castle => SOUND_CASTLE,
            SoundType::Promote => SOUND_PROMOTE,
            SoundType::GameStart => SOUND_GAME_START,
            SoundType::GameEnd => SOUND_GAME_END,
        }
    }
}

pub fn sound_for_san(san: &str) -> SoundType {
    if san.contains('+') || san.contains('#') {
        SoundType::Check
    } else if san.contains('x') {
        SoundType::Capture
    } else if san.contains("O-O") || san.contains("0-0") {
        SoundType::Castle
    } else if san.contains('=') {
        SoundType::Promote
    } else {
        SoundType::Move
    }
}

pub fn sound_for_move(m: &Move, next_pos: &Chess) -> SoundType {
    if next_pos.is_check() || next_pos.is_checkmate() {
        SoundType::Check
    } else if m.is_capture() {
        SoundType::Capture
    } else if m.is_castle() {
        SoundType::Castle
    } else if m.is_promotion() {
        SoundType::Promote
    } else {
        SoundType::Move
    }
}

static SOUND_TX: OnceLock<Sender<SoundType>> = OnceLock::new();

fn get_sound_tx() -> &'static Sender<SoundType> {
    SOUND_TX.get_or_init(|| {
        let (tx, rx) = channel::<SoundType>();
        let _ = std::thread::Builder::new()
            .name("sound-worker".to_string())
            .spawn(move || {
                let stream_res = OutputStream::try_default();
                let (_stream, handle) = match stream_res {
                    Ok((s, h)) => (Some(s), Some(h)),
                    Err(e) => {
                        eprintln!("[audio] No audio output device available: {:?}", e);
                        (None, None)
                    }
                };

                while let Ok(sound) = rx.recv() {
                    if let Some(ref h) = handle {
                        let bytes = sound.bytes();
                        if let Ok(decoder) = Decoder::new(Cursor::new(bytes)) {
                            let _ = h.play_raw(decoder.convert_samples());
                        }
                    }
                }
            });
        tx
    })
}

/// Plays a Chess.com sound effect asynchronously in the background.
pub fn play_sound(sound: SoundType) {
    let _ = get_sound_tx().send(sound);
}
