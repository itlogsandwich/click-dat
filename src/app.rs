use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::Duration;

use eframe::{NativeOptions, Renderer, egui};

const DEFAULT_CLICKS_PER_SECOND: u64 = 1;
const MIN_CLICKS_PER_SECOND: u64 = 1;

pub struct SharedState {
    clicks_per_second: AtomicU64,
    is_clicking: AtomicBool,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            clicks_per_second: AtomicU64::new(DEFAULT_CLICKS_PER_SECOND),
            is_clicking: AtomicBool::new(false),
        }
    }
}

impl SharedState {
    pub fn clicks_per_second(&self) -> u64 {
        self.clicks_per_second.load(Ordering::Relaxed)
    }

    pub fn increment_clicks_per_second(&self) {
        self.clicks_per_second.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_clicks_per_second(&self) {
        let _ = self
            .clicks_per_second
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |clicks| {
                (clicks > MIN_CLICKS_PER_SECOND).then_some(clicks - 1)
            });
    }

    pub fn is_clicking(&self) -> bool {
        self.is_clicking.load(Ordering::Relaxed)
    }

    pub fn toggle_clicking(&self) {
        self.is_clicking.fetch_xor(true, Ordering::Relaxed);
    }

    pub fn click_interval(&self) -> Duration {
        click_interval_for_clicks_per_second(self.clicks_per_second())
    }
}

pub struct AppState {
    state: Arc<SharedState>,
}

impl AppState {
    pub fn new(state: Arc<SharedState>) -> Self {
        Self { state }
    }
}

impl eframe::App for AppState {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().request_repaint_after(Duration::from_millis(100));

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("click dat");
            ui.horizontal(|ui| {
                ui.label("Clicks/sec");
                ui.label(self.state.clicks_per_second().to_string());
            });
            ui.horizontal(|ui| {
                ui.label("Clicking");
                ui.label(if self.state.is_clicking() {
                    "on"
                } else {
                    "off"
                });
            });
            ui.horizontal(|ui| {
                ui.label("Press F6 to turn On/Off");
            });
            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    self.state.increment_clicks_per_second();
                }
                if ui.button("-").clicked() {
                    self.state.decrement_clicks_per_second();
                }
            });
            ui.label("itlogsandwich 2026");
        });
    }
}

pub fn options(width: f32, height: f32) -> NativeOptions {
    NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([width, height]),
        renderer: Renderer::Glow,
        ..Default::default()
    }
}

pub fn click_interval_for_clicks_per_second(clicks_per_second: u64) -> Duration {
    Duration::from_secs_f64(1.0 / clicks_per_second.max(MIN_CLICKS_PER_SECOND) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_state_defaults_to_one_click_per_second() {
        let state = SharedState::default();

        assert_eq!(state.clicks_per_second(), 1);
    }

    #[test]
    fn clicks_per_second_can_be_incremented_and_not_decremented_below_one() {
        let state = SharedState::default();

        state.increment_clicks_per_second();
        assert_eq!(state.clicks_per_second(), 2);

        state.decrement_clicks_per_second();
        state.decrement_clicks_per_second();
        assert_eq!(state.clicks_per_second(), 1);
    }

    #[test]
    fn click_interval_duration_is_derived_from_clicks_per_second() {
        assert_eq!(
            click_interval_for_clicks_per_second(1),
            Duration::from_millis(1000)
        );
        assert_eq!(
            click_interval_for_clicks_per_second(2),
            Duration::from_millis(500)
        );
        assert_eq!(
            click_interval_for_clicks_per_second(10),
            Duration::from_millis(100)
        );
    }
}
