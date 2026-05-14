use enigo::{Enigo, Mouse, Settings, Direction::Click, Button };
use inputbot::{ handle_input_events, KeybdKey::F6Key };
use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, thread::sleep, time::Duration, thread };

fn main() {
    println!("Oh im clicking it fr");

    let is_clicking = Arc::new(AtomicBool::new(false));

    let is_clicking_clone = is_clicking.clone();

    thread::spawn( move ||{

        let mut enigo = Enigo::new(&Settings::default()).expect("failed to initialize");

        loop {
            if is_clicking_clone.load(Ordering::Relaxed) {
                let _ = enigo.button(Button::Left, Click).expect("Failed to click");
                sleep(Duration::from_millis(1));
            }
        }
    });

    F6Key.bind(move || {
        is_clicking.store(!is_clicking.load(Ordering::Relaxed), Ordering::Relaxed);
    });

    handle_input_events();
}
