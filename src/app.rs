use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::Duration;

use eframe::{NativeOptions, egui};

const DEFAULT_INTERVAL_SECS: u64 = 1;
const MIN_INTERVAL_SECS: u64 = 1;

pub struct SharedState {
    interval_secs: AtomicU64,
    is_clicking: AtomicBool,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            interval_secs: AtomicU64::new(DEFAULT_INTERVAL_SECS),
            is_clicking: AtomicBool::new(false),
        }
    }
}

impl SharedState {
    pub fn interval_secs(&self) -> u64 {
        self.interval_secs.load(Ordering::Relaxed)
    }

    pub fn increment_interval(&self) {
        self.interval_secs.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_interval(&self) {
        let _ = self
            .interval_secs
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |interval| {
                (interval > MIN_INTERVAL_SECS).then_some(interval - 1)
            });
    }

    pub fn is_clicking(&self) -> bool {
        self.is_clicking.load(Ordering::Relaxed)
    }

    pub fn toggle_clicking(&self) {
        self.is_clicking.fetch_xor(true, Ordering::Relaxed);
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
                ui.label("Interval");
                ui.label(format!("{}s", self.state.interval_secs()));
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
                    self.state.increment_interval();
                }
                if ui.button("-").clicked() {
                    self.state.decrement_interval();
                }
            });
            ui.label("itlogsandwich 2026");
        });
    }
}

pub fn options(width: f32, height: f32) -> NativeOptions {
    NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([width, height]),
        ..Default::default()
    }
}
