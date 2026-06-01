#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use enigo::{Button, Direction::Click, Enigo, Mouse, Settings};
use inputbot::{KeybdKey::F6Key, handle_input_events};
use std::{
    sync::Arc,
    thread,
    thread::sleep,
    time::{Duration, Instant},
};

mod app;

fn main() {
    env_logger::init();

    let shared_state = Arc::new(app::SharedState::default());

    let click_state = shared_state.clone();

    thread::spawn(move || {
        let mut enigo = Enigo::new(&Settings::default()).expect("failed to initialize");

        loop {
            if click_state.is_clicking() {
                let _ = enigo.button(Button::Left, Click).expect("Failed to click");
                sleep_for_current_interval(&click_state);
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
        app::options(300.0, 300.0),
        Box::new(|_| Ok(Box::new(app::AppState::new(shared_state)))),
    ) {
        println!("Oh nooooo, {error}");
    }
}

fn sleep_for_current_interval(state: &app::SharedState) {
    let started_at = Instant::now();

    while state.is_clicking() {
        let target = Duration::from_secs(state.interval_secs());
        let elapsed = started_at.elapsed();

        if elapsed >= target {
            break;
        }

        let remaining = target - elapsed;
        sleep(remaining.min(Duration::from_millis(50)));
    }
}
