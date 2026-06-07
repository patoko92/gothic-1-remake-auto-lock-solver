use enigo::{Direction as EnigoDirection, Enigo, Key, Keyboard, Settings};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use crate::solver::{Direction, Move};

const KEY_DELAY_MS: u64 = 50;
const LAYER_SWITCH_DELAY_MS: u64 = 80;

/// Play a sequence of moves as WASD keyboard input.
/// Checks `playing` flag before each keystroke — stops early if set to false.
///
/// Mapping:
///   W = next layer (layer index +1)
///   S = previous layer (layer index -1)
///   A = Left  (+1 position)
///   D = Right (-1 position)
///
/// We always start navigation from layer 0.
pub fn play_moves(moves: &[Move], playing: &AtomicBool) {
    let settings = Settings::default();
    let mut enigo = match Enigo::new(&settings) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut current_layer: usize = 0;

    for mv in moves {
        if !playing.load(Ordering::SeqCst) {
            break;
        }

        let target_layer = mv.bar;

        // Navigate to target layer via W/S
        while current_layer < target_layer {
            if !playing.load(Ordering::SeqCst) {
                break;
            }
            let _ = enigo.key(Key::Unicode('w'), EnigoDirection::Click);
            thread::sleep(Duration::from_millis(KEY_DELAY_MS));
            current_layer += 1;
        }
        while current_layer > target_layer {
            if !playing.load(Ordering::SeqCst) {
                break;
            }
            let _ = enigo.key(Key::Unicode('s'), EnigoDirection::Click);
            thread::sleep(Duration::from_millis(KEY_DELAY_MS));
            current_layer -= 1;
        }

        if !playing.load(Ordering::SeqCst) {
            break;
        }

        thread::sleep(Duration::from_millis(LAYER_SWITCH_DELAY_MS));

        // Press direction key A or D
        if playing.load(Ordering::SeqCst) {
            let key = match mv.direction {
                Direction::Left => Key::Unicode('a'),
                Direction::Right => Key::Unicode('d'),
            };
            let _ = enigo.key(key, EnigoDirection::Click);
            thread::sleep(Duration::from_millis(KEY_DELAY_MS));
        }
    }
}


