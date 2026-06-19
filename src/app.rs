use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::Duration;

use eframe::{NativeOptions, Renderer, egui};

const DEFAULT_CLICKS_PER_SECOND: u64 = 1;
const MIN_CLICKS_PER_SECOND: u64 = 1;
const MAX_CLICKS_PER_SECOND: u64 = 50;
const TOGGLE_BUTTON_WIDTH: f32 = 180.0;

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

    pub fn set_clicks_per_second(&self, clicks_per_second: u64) {
        self.clicks_per_second.store(
            clamp_clicks_per_second(clicks_per_second),
            Ordering::Relaxed,
        );
    }

    pub fn increment_clicks_per_second(&self) {
        let _ =
            self.clicks_per_second
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |clicks| {
                    (clicks < MAX_CLICKS_PER_SECOND).then_some(clicks + 1)
                });
    }

    pub fn decrement_clicks_per_second(&self) {
        let _ =
            self.clicks_per_second
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
    style_configured: bool,
}

impl AppState {
    pub fn new(state: Arc<SharedState>) -> Self {
        Self {
            state,
            style_configured: false,
        }
    }

    fn configure_style_once(&mut self, ctx: &egui::Context) {
        if self.style_configured {
            return;
        }

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(18, 20, 24);
        visuals.window_fill = egui::Color32::from_rgb(24, 27, 32);
        visuals.extreme_bg_color = egui::Color32::from_rgb(12, 14, 17);
        visuals.faint_bg_color = egui::Color32::from_rgb(35, 39, 46);
        visuals.selection.bg_fill = egui::Color32::from_rgb(67, 132, 255);
        visuals.slider_trailing_fill = true;
        ctx.set_visuals(visuals);

        ctx.global_style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(12.0, 8.0);
            style.spacing.slider_width = 210.0;
            style.spacing.interact_size = egui::vec2(40.0, 34.0);
        });

        self.style_configured = true;
    }
}

impl eframe::App for AppState {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.configure_style_once(ui.ctx());
        ui.ctx().request_repaint_after(Duration::from_millis(100));

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(18, 20, 24))
                    .inner_margin(18),
            )
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("click dat").size(22.0).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        status_pill(ui, self.state.is_clicking());
                    });
                });

                ui.add_space(6.0);

                let clicks_per_second = self.state.clicks_per_second();
                ui.label(
                    egui::RichText::new(format!("{clicks_per_second} CPS"))
                        .size(34.0)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(click_interval_label_for_clicks_per_second(
                        clicks_per_second,
                    ))
                    .color(egui::Color32::from_rgb(167, 174, 186)),
                );

                ui.add_space(8.0);

                let mut slider_clicks_per_second = clicks_per_second;
                let slider_response = ui.add(
                    egui::Slider::new(
                        &mut slider_clicks_per_second,
                        MIN_CLICKS_PER_SECOND..=MAX_CLICKS_PER_SECOND,
                    )
                    .text("Clicks/sec")
                    .suffix(" cps")
                    .step_by(1.0),
                );
                if slider_response.changed() {
                    self.state.set_clicks_per_second(slider_clicks_per_second);
                }

                ui.horizontal(|ui| {
                    if ui
                        .add_sized([44.0, 34.0], egui::Button::new("-"))
                        .on_hover_text("Decrease by 1 CPS")
                        .clicked()
                    {
                        self.state.decrement_clicks_per_second();
                    }
                    if ui
                        .add_sized([44.0, 34.0], egui::Button::new("+"))
                        .on_hover_text("Increase by 1 CPS")
                        .clicked()
                    {
                        self.state.increment_clicks_per_second();
                    }
                    let toggle_text = if self.state.is_clicking() {
                        "Stop"
                    } else {
                        "Start"
                    };
                    let toggle_fill = if self.state.is_clicking() {
                        egui::Color32::from_rgb(170, 64, 64)
                    } else {
                        egui::Color32::from_rgb(54, 128, 86)
                    };
                    if ui
                        .add_sized(
                            [TOGGLE_BUTTON_WIDTH, 34.0],
                            egui::Button::new(egui::RichText::new(toggle_text).strong())
                                .fill(toggle_fill),
                        )
                        .on_hover_text("Toggle clicking")
                        .clicked()
                    {
                        self.state.toggle_clicking();
                    }
                });

                ui.add_space(4.0);
                ui.separator();
                ui.label(
                    egui::RichText::new("F6 toggles clicking")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(141, 149, 163)),
                );
            });
    }
}

fn status_pill(ui: &mut egui::Ui, is_clicking: bool) {
    let (label, fill, text) = if is_clicking {
        (
            "ON",
            egui::Color32::from_rgb(36, 93, 60),
            egui::Color32::from_rgb(187, 245, 208),
        )
    } else {
        (
            "OFF",
            egui::Color32::from_rgb(63, 68, 78),
            egui::Color32::from_rgb(207, 213, 224),
        )
    };

    egui::Frame::new()
        .fill(fill)
        .corner_radius(6)
        .inner_margin(egui::Margin::symmetric(9, 4))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(label).size(12.0).strong().color(text));
        });
}

pub fn options(width: f32, height: f32) -> NativeOptions {
    NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_min_inner_size([320.0, 220.0]),
        renderer: Renderer::Glow,
        ..Default::default()
    }
}

pub fn click_interval_for_clicks_per_second(clicks_per_second: u64) -> Duration {
    Duration::from_secs_f64(1.0 / clamp_clicks_per_second(clicks_per_second) as f64)
}

pub fn click_interval_label_for_clicks_per_second(clicks_per_second: u64) -> String {
    let interval_millis = click_interval_for_clicks_per_second(clicks_per_second).as_millis();
    format!("1 click every {interval_millis} ms")
}

fn clamp_clicks_per_second(clicks_per_second: u64) -> u64 {
    clicks_per_second.clamp(MIN_CLICKS_PER_SECOND, MAX_CLICKS_PER_SECOND)
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
    fn clicks_per_second_can_be_set_within_bounds() {
        let state = SharedState::default();

        state.set_clicks_per_second(25);

        assert_eq!(state.clicks_per_second(), 25);
    }

    #[test]
    fn clicks_per_second_setter_clamps_to_supported_range() {
        let state = SharedState::default();

        state.set_clicks_per_second(0);
        assert_eq!(state.clicks_per_second(), 1);

        state.set_clicks_per_second(999);
        assert_eq!(state.clicks_per_second(), 50);
    }

    #[test]
    fn clicks_per_second_increment_does_not_exceed_maximum() {
        let state = SharedState::default();
        state.set_clicks_per_second(50);

        state.increment_clicks_per_second();

        assert_eq!(state.clicks_per_second(), 50);
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
        assert_eq!(
            click_interval_for_clicks_per_second(50),
            Duration::from_millis(20)
        );
    }

    #[test]
    fn click_interval_label_describes_derived_timing() {
        assert_eq!(
            click_interval_label_for_clicks_per_second(1),
            "1 click every 1000 ms"
        );
        assert_eq!(
            click_interval_label_for_clicks_per_second(50),
            "1 click every 20 ms"
        );
    }
}
