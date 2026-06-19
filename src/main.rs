#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use enigo::{Button, Direction::Click, Enigo, Mouse, Settings};
use inputbot::{KeybdKey::F6Key, handle_input_events};
use std::{
    sync::Arc,
    thread,
    thread::{sleep, yield_now},
    time::{Duration, Instant},
};

mod app;

const CLICK_TIMING_CUSHION: Duration = Duration::from_millis(1);
const MAX_SLEEP_SLICE: Duration = Duration::from_millis(2);

fn main() {
    env_logger::init();

    let shared_state = Arc::new(app::SharedState::default());

    let click_state = shared_state.clone();

    thread::spawn(move || {
        let mut enigo = Enigo::new(&Settings::default()).expect("failed to initialize");

        loop {
            if click_state.is_clicking() {
                let clicked_at = Instant::now();
                let _ = enigo.button(Button::Left, Click).expect("Failed to click");
                sleep_for_current_interval(&click_state, clicked_at);
            } else {
                sleep(Duration::from_millis(50));
            }
        }
    });

    let hotkey_state = shared_state.clone();
    F6Key.bind(move || {
        hotkey_state.toggle_clicking();
    });

    thread::spawn(handle_input_events);

    if let Err(error) = eframe::run_native(
        "click dat",
        app::options(380.0, 280.0),
        Box::new(|_| Ok(Box::new(app::AppState::new(shared_state)))),
    ) {
        println!("Oh nooooo, {error}");
    }
}

fn sleep_for_current_interval(state: &app::SharedState, clicked_at: Instant) {
    while state.is_clicking() {
        let remaining = remaining_click_interval(state.click_interval(), clicked_at.elapsed());

        if remaining.is_zero() {
            break;
        }

        if remaining > CLICK_TIMING_CUSHION {
            sleep((remaining - CLICK_TIMING_CUSHION).min(MAX_SLEEP_SLICE));
        } else {
            yield_now();
        }
    }
}

fn remaining_click_interval(target: Duration, elapsed: Duration) -> Duration {
    target
        .saturating_sub(CLICK_TIMING_CUSHION)
        .saturating_sub(elapsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaining_click_interval_counts_click_elapsed_time() {
        assert_eq!(
            remaining_click_interval(Duration::from_millis(20), Duration::from_millis(4)),
            Duration::from_millis(15)
        );
    }

    #[test]
    fn remaining_click_interval_is_zero_when_click_exceeds_target() {
        assert_eq!(
            remaining_click_interval(Duration::from_millis(20), Duration::from_millis(25)),
            Duration::ZERO
        );
    }
}
