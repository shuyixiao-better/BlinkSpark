#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::{
    fs, io,
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::{Parser, ValueEnum};
use eframe::egui;

#[cfg(target_os = "windows")]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex, OnceLock,
};

mod notifier;

const WINDOW_SIZE: egui::Vec2 = egui::vec2(420.0, 290.0);
const WINDOW_MIN_SIZE: egui::Vec2 = egui::vec2(380.0, 270.0);

const PANEL_PADDING: i8 = 20;
const CARD_PADDING: i8 = 18;
const CARD_RADIUS: u8 = 20;
const CONTROL_RADIUS: u8 = 14;
const CONTROL_HEIGHT: f32 = 30.0;
const RESET_BUTTON_WIDTH: f32 = 156.0;
const DESKTOP_PIN_BUTTON_WIDTH: f32 = 132.0;
const PROGRESS_HEIGHT: f32 = 16.0;
const PROGRESS_ANIMATION_SECONDS: f32 = 0.5;

const SPACE_XS: f32 = 4.0;
const SPACE_SM: f32 = 8.0;
const FOOTER_HEIGHT: f32 = 52.0;

const COLOR_BACKGROUND: (u8, u8, u8) = (245, 247, 250);
const COLOR_PRIMARY: (u8, u8, u8) = (142, 182, 155);
const COLOR_PRIMARY_GRADIENT_END: (u8, u8, u8) = (114, 159, 128);
const COLOR_TITLE: (u8, u8, u8) = (37, 52, 63);
const COLOR_TEXT_MUTED: (u8, u8, u8) = (97, 112, 129);
const COLOR_TEXT_HINT: (u8, u8, u8) = (117, 130, 146);
const COLOR_ERROR: (u8, u8, u8) = (195, 56, 56);
const COLOR_WHITE: (u8, u8, u8) = (255, 255, 255);

fn rgb((r, g, b): (u8, u8, u8)) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}

fn rgba((r, g, b): (u8, u8, u8), alpha: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(r, g, b, alpha)
}

#[derive(Parser, Debug, Clone, Copy)]
#[command(name = "blinkspark", version, about = "20-20-20 blink reminder")]
struct Args {
    /// Reminder language: zh or en
    #[arg(long, value_enum, default_value_t = Lang::En)]
    lang: Lang,

    /// Interval in minutes before sending the reminder
    #[arg(long, default_value_t = 20)]
    interval: u64,

    /// Send one reminder and exit
    #[arg(long, default_value_t = false)]
    once: bool,

    /// Repeat reminders every interval until interrupted (legacy option; now default)
    #[arg(long, default_value_t = false, hide = true)]
    repeat: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
enum Lang {
    Zh,
    En,
}

impl Lang {
    fn message(self) -> (&'static str, &'static str) {
        match self {
            Lang::Zh => (
                "\u{8BE5}\u{7728}\u{773C}\u{4E86}",
                "\u{9075}\u{5FAA} 20-20-20 \u{89C4}\u{5219}\u{FF1A}\u{770B} 6 \u{7C73}\u{5916} 20 \u{79D2}\u{3002}",
            ),
            Lang::En => (
                "Time to blink",
                "Follow the 20-20-20 rule: look 20 feet away for 20 seconds.",
            ),
        }
    }

    fn window_title(self) -> &'static str {
        match self {
            Lang::Zh => "\u{7728}\u{773C}\u{63D0}\u{9192}",
            Lang::En => "Blink Reminder",
        }
    }

    fn countdown_label(self) -> &'static str {
        match self {
            Lang::Zh => "\u{4E0B}\u{6B21}\u{63D0}\u{9192}\u{5012}\u{8BA1}\u{65F6}",
            Lang::En => "Next reminder in",
        }
    }

    fn mode_label(self, once_mode: bool) -> &'static str {
        match (self, once_mode) {
            (Lang::Zh, true) => "\u{6A21}\u{5F0F}\u{FF1A}\u{5355}\u{6B21}\u{63D0}\u{9192}",
            (Lang::Zh, false) => "\u{6A21}\u{5F0F}\u{FF1A}\u{5FAA}\u{73AF}\u{63D0}\u{9192}",
            (Lang::En, true) => "Mode: once",
            (Lang::En, false) => "Mode: repeat",
        }
    }

    fn drag_hint(self) -> &'static str {
        match self {
            Lang::Zh => {
                "\u{62D6}\u{52A8}\u{6B64}\u{7A97}\u{53E3}\u{5230}\u{4EFB}\u{610F}\u{4F4D}\u{7F6E}"
            }
            Lang::En => "Drag this window to place it anywhere",
        }
    }

    fn language_label(self) -> &'static str {
        match self {
            Lang::Zh => "\u{8BED}\u{8A00}",
            Lang::En => "Language",
        }
    }

    fn reset_button_label(self) -> &'static str {
        match self {
            Lang::Zh => "\u{91CD}\u{7F6E}\u{8BA1}\u{65F6}",
            Lang::En => "Reset timer",
        }
    }

    fn pin_button_label(self, pinned: bool) -> &'static str {
        match (self, pinned) {
            (Lang::Zh, true) => "\u{53D6}\u{6D88}\u{9489}\u{684C}\u{9762}",
            (Lang::Zh, false) => "\u{9489}\u{5728}\u{684C}\u{9762}",
            (Lang::En, true) => "Unpin desktop",
            (Lang::En, false) => "Pin to desktop",
        }
    }

    fn notify_error_prefix(self) -> &'static str {
        match self {
            Lang::Zh => "\u{901A}\u{77E5}\u{5931}\u{8D25}\u{FF1A}",
            Lang::En => "Notification failed:",
        }
    }

    fn position_error_prefix(self) -> &'static str {
        match self {
            Lang::Zh => "\u{4FDD}\u{5B58}\u{7A97}\u{53E3}\u{4F4D}\u{7F6E}\u{5931}\u{8D25}\u{FF1A}",
            Lang::En => "Failed to save window position:",
        }
    }

    fn pin_error_prefix(self) -> &'static str {
        match self {
            Lang::Zh => "\u{9489}\u{684C}\u{9762}\u{5904}\u{7406}\u{5931}\u{8D25}\u{FF1A}",
            Lang::En => "Failed to apply desktop pin mode:",
        }
    }
}

struct CountdownApp {
    interval: Duration,
    next_deadline: Instant,
    lang: Lang,
    once_mode: bool,
    position_path: PathBuf,
    desktop_pin_path: PathBuf,
    last_saved_position: Option<egui::Pos2>,
    last_position_save_at: Option<Instant>,
    last_visibility_recover_at: Option<Instant>,
    last_error: Option<String>,
    last_position_error: Option<String>,
    pin_to_desktop: bool,
    last_applied_pin_to_desktop: Option<bool>,
    last_pin_error: Option<String>,
    displayed_progress: f32,
    finished: bool,
}

impl CountdownApp {
    fn new(
        args: Args,
        initial_saved_position: Option<egui::Pos2>,
        initial_pin_to_desktop: bool,
    ) -> Self {
        let interval = Duration::from_secs(args.interval.saturating_mul(60));
        Self {
            interval,
            next_deadline: Instant::now() + interval,
            lang: args.lang,
            once_mode: args.once && !args.repeat,
            position_path: window_position_file(),
            desktop_pin_path: desktop_pin_file(),
            last_saved_position: initial_saved_position,
            last_position_save_at: None,
            last_visibility_recover_at: None,
            last_error: None,
            last_position_error: None,
            pin_to_desktop: initial_pin_to_desktop,
            last_applied_pin_to_desktop: None,
            last_pin_error: None,
            displayed_progress: 1.0,
            finished: false,
        }
    }

    fn try_send_notification(&mut self) {
        let now = Instant::now();
        if now < self.next_deadline {
            return;
        }

        let (title, body) = self.lang.message();
        if let Err(err) = notifier::notify(title, body) {
            self.last_error = Some(err);
        } else {
            self.last_error = None;
        }

        if self.once_mode {
            self.finished = true;
        } else {
            self.roll_deadline_forward(now);
        }
    }

    fn roll_deadline_forward(&mut self, now: Instant) {
        while self.next_deadline <= now {
            self.next_deadline += self.interval;
        }
        self.displayed_progress = 1.0;
    }

    fn reset_timer(&mut self) {
        self.next_deadline = Instant::now() + self.interval;
        self.displayed_progress = 1.0;
    }

    fn apply_desktop_pin_mode(&mut self, ctx: &egui::Context) {
        if self.last_applied_pin_to_desktop == Some(self.pin_to_desktop) {
            return;
        }

        #[cfg(target_os = "windows")]
        if self.pin_to_desktop {
            // Windows desktop-pinned windows are reparented and cannot reliably stay maximized.
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
        }

        let level = if self.pin_to_desktop {
            #[cfg(target_os = "windows")]
            {
                egui::WindowLevel::Normal
            }
            #[cfg(not(target_os = "windows"))]
            {
                egui::WindowLevel::AlwaysOnBottom
            }
        } else {
            egui::WindowLevel::Normal
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));

        #[cfg(target_os = "windows")]
        let native_pin_result = {
            set_windows_desktop_pin_enabled(self.pin_to_desktop);
            sync_windows_desktop_pin(self.pin_to_desktop)
        };

        self.last_pin_error = match save_desktop_pin(&self.desktop_pin_path, self.pin_to_desktop) {
            Ok(()) => {
                #[cfg(target_os = "windows")]
                {
                    native_pin_result.err()
                }
                #[cfg(not(target_os = "windows"))]
                {
                    None
                }
            }
            Err(err) => Some(err.to_string()),
        };
        self.last_applied_pin_to_desktop = Some(self.pin_to_desktop);
    }

    fn maybe_persist_window_position(&mut self, ctx: &egui::Context, force: bool) {
        let Some(current_pos) = ctx.input(|i| i.viewport().outer_rect.map(|rect| rect.min)) else {
            return;
        };

        let moved = self
            .last_saved_position
            .is_none_or(|last| position_changed(last, current_pos));

        if !moved && !force {
            return;
        }

        if !force {
            let save_too_soon = self
                .last_position_save_at
                .is_some_and(|last| last.elapsed() < Duration::from_millis(250));
            if save_too_soon {
                return;
            }
        }

        match save_window_position(&self.position_path, current_pos) {
            Ok(()) => {
                self.last_saved_position = Some(current_pos);
                self.last_position_save_at = Some(Instant::now());
                self.last_position_error = None;
            }
            Err(err) => {
                self.last_position_error = Some(err.to_string());
            }
        }
    }

    fn ensure_window_visible(&mut self, ctx: &egui::Context) {
        let Some(visible_rect) = platform_visible_rect() else {
            return;
        };

        let Some(window_rect) = ctx.input(|i| i.viewport().outer_rect) else {
            return;
        };

        // If any part of the window is still visible, keep the user's current placement.
        if window_rect.intersects(visible_rect) {
            return;
        }

        let recover_too_soon = self
            .last_visibility_recover_at
            .is_some_and(|last| last.elapsed() < Duration::from_secs(1));
        if recover_too_soon {
            return;
        }

        let corrected = clamp_window_position(window_rect.min, window_rect.size());
        if !position_changed(window_rect.min, corrected) {
            return;
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(corrected));
        self.last_visibility_recover_at = Some(Instant::now());
        self.last_saved_position = Some(corrected);
        self.last_position_save_at = Some(Instant::now());

        if let Err(err) = save_window_position(&self.position_path, corrected) {
            self.last_position_error = Some(err.to_string());
        } else {
            self.last_position_error = None;
        }
    }
}

impl eframe::App for CountdownApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_desktop_pin_mode(ctx);
        self.ensure_window_visible(ctx);
        self.maybe_persist_window_position(ctx, false);

        if ctx.input(|i| i.viewport().close_requested()) {
            self.maybe_persist_window_position(ctx, true);
        }

        if self.finished {
            self.maybe_persist_window_position(ctx, true);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.try_send_notification();
        let remaining = self.next_deadline.saturating_duration_since(Instant::now());
        let target_progress = countdown_progress(self.interval, remaining);
        let dt = ctx.input(|i| i.stable_dt.max(1.0 / 240.0));
        self.displayed_progress = animate_value(
            self.displayed_progress,
            target_progress,
            dt,
            PROGRESS_ANIMATION_SECONDS,
        );

        let animating = (self.displayed_progress - target_progress).abs() > 0.001;
        ctx.request_repaint_after(if animating {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(120)
        });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::TRANSPARENT)
                    .inner_margin(egui::Margin::same(PANEL_PADDING)),
            )
            .show(ctx, |ui| {
                paint_glass_background(ui);

                egui::Frame::default()
                    .fill(rgba(COLOR_BACKGROUND, 220))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 13),
                    ))
                    .corner_radius(egui::CornerRadius::same(CARD_RADIUS))
                    .inner_margin(egui::Margin::same(CARD_PADDING))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(letter_spaced_title(
                                        self.lang.window_title(),
                                    ))
                                    .size(24.0)
                                    .color(rgb(COLOR_TITLE)),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(
                                                self.lang.mode_label(self.once_mode),
                                            )
                                            .size(13.0)
                                            .color(rgb(COLOR_TEXT_MUTED)),
                                        );
                                    },
                                );
                            });
                            ui.add_space(SPACE_XS);
                            ui.separator();
                            ui.add_space(SPACE_SM);

                            egui::Frame::default()
                                .fill(rgba((255, 255, 255), 170))
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 13),
                                ))
                                .corner_radius(egui::CornerRadius::same(CARD_RADIUS))
                                .inner_margin(egui::Margin::same(CARD_PADDING))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(self.lang.countdown_label())
                                            .size(14.0)
                                            .color(rgb(COLOR_TEXT_MUTED)),
                                    );
                                    ui.add_space(SPACE_XS);
                                    ui.label(
                                        egui::RichText::new(format_countdown(remaining))
                                            .size(48.0)
                                            .monospace()
                                            .color(rgb(COLOR_TITLE)),
                                    );
                                    ui.add_space(SPACE_SM);
                                    draw_gradient_progress_bar(ui, self.displayed_progress);
                                });

                            ui.add_space(SPACE_SM);
                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(self.lang.language_label())
                                            .size(13.0)
                                            .color(rgb(COLOR_TEXT_MUTED)),
                                    );
                                    ui.add_space(SPACE_XS);
                                    segmented_language_control(ui, &mut self.lang);
                                },
                            );

                            ui.add_space(SPACE_XS);
                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    if desktop_pin_button(
                                        ui,
                                        self.lang.pin_button_label(self.pin_to_desktop),
                                        self.pin_to_desktop,
                                    )
                                    .clicked()
                                    {
                                        self.pin_to_desktop = !self.pin_to_desktop;
                                        self.apply_desktop_pin_mode(ctx);
                                    }

                                    let spacer =
                                        (ui.available_width() - RESET_BUTTON_WIDTH).max(SPACE_SM);
                                    ui.add_space(spacer);

                                    if primary_reset_button(ui, self.lang).clicked() {
                                        self.reset_timer();
                                    }
                                },
                            );

                            if let Some(err) = &self.last_error {
                                ui.add_space(SPACE_SM);
                                ui.colored_label(
                                    rgb(COLOR_ERROR),
                                    format!("{} {}", self.lang.notify_error_prefix(), err),
                                );
                            }

                            if let Some(err) = &self.last_position_error {
                                ui.add_space(SPACE_SM);
                                ui.colored_label(
                                    rgb(COLOR_ERROR),
                                    format!("{} {}", self.lang.position_error_prefix(), err),
                                );
                            }

                            if let Some(err) = &self.last_pin_error {
                                ui.add_space(SPACE_SM);
                                ui.colored_label(
                                    rgb(COLOR_ERROR),
                                    format!("{} {}", self.lang.pin_error_prefix(), err),
                                );
                            }

                            ui.add_space(footer_spacer(ui.available_height(), FOOTER_HEIGHT));
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(self.lang.drag_hint())
                                        .size(12.0)
                                        .color(rgb(COLOR_TEXT_HINT)),
                                )
                                .truncate(),
                            );
                        });
                    });
            });
    }
}

fn format_countdown(remaining: Duration) -> String {
    let seconds = remaining.as_secs();
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{secs:02}")
    } else {
        format!("{minutes:02}:{secs:02}")
    }
}

fn countdown_progress(interval: Duration, remaining: Duration) -> f32 {
    let total = interval.as_secs_f32().max(1.0);
    let left = remaining.as_secs_f32().clamp(0.0, total);
    (left / total).clamp(0.0, 1.0)
}

fn footer_spacer(available_height: f32, footer_height: f32) -> f32 {
    (available_height - footer_height).max(0.0)
}

fn lerp_u8(from: u8, to: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    (from as f32 + (to as f32 - from as f32) * t).round() as u8
}

fn lerp_color(from: egui::Color32, to: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    egui::Color32::from_rgba_unmultiplied(
        lerp_u8(from.r(), to.r(), t),
        lerp_u8(from.g(), to.g(), t),
        lerp_u8(from.b(), to.b(), t),
        lerp_u8(from.a(), to.a(), t),
    )
}

fn ease_in_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

fn animate_value(current: f32, target: f32, dt_seconds: f32, duration_seconds: f32) -> f32 {
    let delta = (target - current).abs();
    if delta <= 0.0001 {
        return target;
    }

    let step = (dt_seconds / duration_seconds.max(0.0001)).clamp(0.0, 1.0);
    current + (target - current) * ease_in_out_cubic(step)
}

fn letter_spaced_title(title: &str) -> String {
    let mut result = String::with_capacity(title.len() * 2);
    let mut chars = title.chars().peekable();

    while let Some(current) = chars.next() {
        result.push(current);
        if let Some(next) = chars.peek() {
            if current != ' ' && *next != ' ' {
                result.push(' ');
            }
        }
    }

    result
}

fn paint_glass_background(ui: &mut egui::Ui) {
    let rect = ui.max_rect();
    let painter = ui.painter();

    painter.rect_filled(
        rect,
        egui::CornerRadius::same(CARD_RADIUS),
        rgb(COLOR_BACKGROUND),
    );
    painter.circle_filled(
        rect.left_top() + egui::vec2(rect.width() * 0.28, rect.height() * 0.20),
        rect.width() * 0.42,
        rgba(COLOR_PRIMARY, 28),
    );
    painter.circle_filled(
        rect.right_bottom() - egui::vec2(rect.width() * 0.20, rect.height() * 0.18),
        rect.width() * 0.38,
        rgba((255, 255, 255), 130),
    );
}

fn draw_gradient_progress_bar(ui: &mut egui::Ui, progress: f32) {
    let (track_rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), PROGRESS_HEIGHT),
        egui::Sense::hover(),
    );

    let radius = egui::CornerRadius::same((PROGRESS_HEIGHT / 2.0).round() as u8);
    let track_color = rgba((255, 255, 255), 150);
    ui.painter().rect_filled(track_rect, radius, track_color);
    ui.painter().rect_stroke(
        track_rect,
        radius,
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 13)),
        egui::StrokeKind::Inside,
    );

    let fill_width = track_rect.width() * progress.clamp(0.0, 1.0);
    if fill_width <= 0.0 {
        return;
    }

    let fill_rect =
        egui::Rect::from_min_size(track_rect.min, egui::vec2(fill_width, track_rect.height()));
    let fill_radius =
        egui::CornerRadius::same((fill_width.min(PROGRESS_HEIGHT) / 2.0).round() as u8);
    let gradient_start = rgb(COLOR_PRIMARY);
    let gradient_end = rgb(COLOR_PRIMARY_GRADIENT_END);
    ui.painter()
        .rect_filled(fill_rect, fill_radius, gradient_start);

    let steps: usize = 24;
    for step in 0..steps {
        let t0 = step as f32 / steps as f32;
        let t1 = (step + 1) as f32 / steps as f32;
        let x0 = fill_rect.left() + fill_rect.width() * t0;
        let x1 = fill_rect.left() + fill_rect.width() * t1;
        let segment = egui::Rect::from_min_max(
            egui::pos2(x0, fill_rect.top()),
            egui::pos2(x1, fill_rect.bottom()),
        );
        let color = lerp_color(gradient_start, gradient_end, t0);
        ui.painter().rect_filled(segment, 0.0, color);
    }
}

fn segmented_language_control(ui: &mut egui::Ui, lang: &mut Lang) {
    egui::Frame::default()
        .fill(rgba((255, 255, 255), 185))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 13),
        ))
        .corner_radius(egui::CornerRadius::same(CARD_RADIUS))
        .inner_margin(egui::Margin::symmetric(4, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                segmented_language_item(ui, lang, Lang::En, "EN");
                segmented_language_item(ui, lang, Lang::Zh, "ZH");
            });
        });
}

fn segmented_language_item(ui: &mut egui::Ui, lang: &mut Lang, choice: Lang, label: &str) {
    let selected = *lang == choice;
    let fill = if selected {
        rgba(COLOR_PRIMARY, 180)
    } else {
        egui::Color32::TRANSPARENT
    };
    let text_color = if selected {
        rgb(COLOR_TITLE)
    } else {
        rgb(COLOR_TEXT_MUTED)
    };

    let button = egui::Button::new(egui::RichText::new(label).color(text_color))
        .fill(fill)
        .stroke(egui::Stroke::NONE)
        .corner_radius(egui::CornerRadius::same(CONTROL_RADIUS));
    if ui.add_sized([44.0, CONTROL_HEIGHT], button).clicked() {
        *lang = choice;
    }
}

fn primary_reset_button(ui: &mut egui::Ui, lang: Lang) -> egui::Response {
    let corner = egui::CornerRadius::same((CONTROL_HEIGHT / 2.0).round() as u8);
    ui.scope(|ui| {
        let visuals = ui.visuals_mut();
        visuals.widgets.hovered.bg_fill = rgb(COLOR_PRIMARY_GRADIENT_END);
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, rgb(COLOR_PRIMARY_GRADIENT_END));
        visuals.widgets.active.bg_fill = rgb(COLOR_PRIMARY_GRADIENT_END);
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, rgb(COLOR_PRIMARY_GRADIENT_END));

        let label = egui::RichText::new(format!("\u{21BB} {}", lang.reset_button_label()))
            .size(13.0)
            .strong()
            .color(rgb(COLOR_WHITE));
        let button = egui::Button::new(label)
            .fill(rgb(COLOR_PRIMARY))
            .stroke(egui::Stroke::new(1.0, rgb(COLOR_PRIMARY)))
            .corner_radius(corner);
        ui.add_sized([RESET_BUTTON_WIDTH, CONTROL_HEIGHT], button)
    })
    .inner
}

fn desktop_pin_button(ui: &mut egui::Ui, label: &str, pinned: bool) -> egui::Response {
    let corner = egui::CornerRadius::same((CONTROL_HEIGHT / 2.0).round() as u8);
    let fill = if pinned {
        rgba(COLOR_PRIMARY, 200)
    } else {
        rgba((255, 255, 255), 185)
    };
    let stroke = if pinned {
        egui::Stroke::new(1.0, rgb(COLOR_PRIMARY_GRADIENT_END))
    } else {
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 18))
    };
    let text_color = if pinned {
        rgb(COLOR_TITLE)
    } else {
        rgb(COLOR_TEXT_MUTED)
    };

    let button = egui::Button::new(egui::RichText::new(label).size(13.0).color(text_color))
        .fill(fill)
        .stroke(stroke)
        .corner_radius(corner);
    ui.add_sized([DESKTOP_PIN_BUTTON_WIDTH, CONTROL_HEIGHT], button)
}

#[cfg(target_os = "windows")]
static WINDOWS_DESKTOP_PIN_ENABLED: AtomicBool = AtomicBool::new(false);
#[cfg(target_os = "windows")]
static WINDOWS_DESKTOP_PIN_GUARD: OnceLock<()> = OnceLock::new();
#[cfg(target_os = "windows")]
static WINDOWS_DESKTOP_PIN_STATE: OnceLock<Mutex<WindowsDesktopPinState>> = OnceLock::new();

#[cfg(target_os = "windows")]
#[derive(Default)]
struct WindowsDesktopPinState {
    original_parent: Option<isize>,
    is_pinned: bool,
}

#[cfg(target_os = "windows")]
fn start_windows_desktop_pin_guard() {
    WINDOWS_DESKTOP_PIN_GUARD.get_or_init(|| {
        std::thread::spawn(|| loop {
            if WINDOWS_DESKTOP_PIN_ENABLED.load(Ordering::Relaxed) {
                let _ = sync_windows_desktop_pin(true);
            }
            std::thread::sleep(Duration::from_millis(250));
        });
    });
}

#[cfg(target_os = "windows")]
fn set_windows_desktop_pin_enabled(enabled: bool) {
    WINDOWS_DESKTOP_PIN_ENABLED.store(enabled, Ordering::Relaxed);
}

#[cfg(target_os = "windows")]
fn windows_desktop_pin_state() -> &'static Mutex<WindowsDesktopPinState> {
    WINDOWS_DESKTOP_PIN_STATE.get_or_init(|| Mutex::new(WindowsDesktopPinState::default()))
}

#[cfg(target_os = "windows")]
fn sync_windows_desktop_pin(enable: bool) -> Result<(), String> {
    let hwnd = find_blinkspark_window().ok_or_else(|| "window handle not found".to_string())?;
    if enable {
        pin_window_to_desktop(hwnd)
    } else {
        unpin_window_from_desktop(hwnd)
    }
}

#[cfg(target_os = "windows")]
fn pin_window_to_desktop(hwnd: windows_sys::Win32::Foundation::HWND) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetParent, SetParent, SetWindowPos, ShowWindow, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOSIZE, SWP_SHOWWINDOW, SW_RESTORE,
    };

    let host = find_desktop_host_window().ok_or_else(|| "desktop host not found".to_string())?;
    let current_parent = unsafe { GetParent(hwnd) };
    let already_pinned_to_host = current_parent == host;

    {
        let mut state = windows_desktop_pin_state()
            .lock()
            .map_err(|_| "desktop pin state lock poisoned".to_string())?;

        if !state.is_pinned {
            // SAFETY: hwnd is a valid top-level window handle.
            state.original_parent =
                (!current_parent.is_null()).then_some(current_parent as isize);
            state.is_pinned = true;
        } else if already_pinned_to_host {
            // Already pinned correctly. Avoid repeatedly restoring/reparenting every guard tick.
            return Ok(());
        }
    }

    // SAFETY: hwnd and host are valid HWNDs managed by the current desktop session.
    unsafe {
        SetParent(hwnd, host);
        ShowWindow(hwnd, SW_RESTORE);
        SetWindowPos(
            hwnd,
            HWND_BOTTOM,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn unpin_window_from_desktop(hwnd: windows_sys::Win32::Foundation::HWND) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SetParent, SetWindowPos, ShowWindow, HWND_NOTOPMOST, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOSIZE, SWP_SHOWWINDOW, SW_RESTORE,
    };

    let restore_parent = {
        let mut state = windows_desktop_pin_state()
            .lock()
            .map_err(|_| "desktop pin state lock poisoned".to_string())?;
        state.is_pinned = false;
        state
            .original_parent
            .take()
            .map_or(std::ptr::null_mut(), |hwnd| {
                hwnd as windows_sys::Win32::Foundation::HWND
            })
    };

    // SAFETY: hwnd is valid and restore_parent is either null or a parent HWND captured earlier.
    unsafe {
        SetParent(hwnd, restore_parent);
        ShowWindow(hwnd, SW_RESTORE);
        SetWindowPos(
            hwnd,
            HWND_NOTOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn find_blinkspark_window() -> Option<windows_sys::Win32::Foundation::HWND> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindow, GetWindowThreadProcessId, GW_OWNER,
    };

    #[derive(Default)]
    struct Search {
        pid: u32,
        hwnd: windows_sys::Win32::Foundation::HWND,
    }

    unsafe extern "system" fn enum_window(
        hwnd: windows_sys::Win32::Foundation::HWND,
        lparam: windows_sys::Win32::Foundation::LPARAM,
    ) -> i32 {
        let search = unsafe { &mut *(lparam as *mut Search) };
        let mut pid = 0_u32;
        // SAFETY: hwnd is provided by EnumWindows; pid out pointer is valid.
        unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        // SAFETY: hwnd is valid; querying owner does not mutate memory.
        let owner = unsafe { GetWindow(hwnd, GW_OWNER) };
        if pid == search.pid && owner.is_null() {
            search.hwnd = hwnd;
            return 0;
        }
        1
    }

    let mut search = Search {
        pid: std::process::id(),
        hwnd: std::ptr::null_mut(),
    };
    // SAFETY: Callback and lparam point to live data for the call duration.
    unsafe {
        EnumWindows(Some(enum_window), &mut search as *mut Search as isize);
    }

    (!search.hwnd.is_null()).then_some(search.hwnd)
}

#[cfg(target_os = "windows")]
fn find_desktop_host_window() -> Option<windows_sys::Win32::Foundation::HWND> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FindWindowExW, FindWindowW, SendMessageTimeoutW, SMTO_NORMAL,
    };

    let progman = {
        let class = wide_null("Progman");
        // SAFETY: class is null-terminated and valid for the duration of the call.
        unsafe { FindWindowW(class.as_ptr(), std::ptr::null()) }
    };

    if progman.is_null() {
        return None;
    }

    // SAFETY: `progman` is a valid HWND; this message asks Explorer to ensure a WorkerW host exists.
    unsafe {
        SendMessageTimeoutW(
            progman,
            0x052C,
            0,
            0,
            SMTO_NORMAL,
            1_000,
            std::ptr::null_mut(),
        );
    }

    let workerw = {
        let class = wide_null("WorkerW");
        // SAFETY: class is null-terminated and valid for the duration of the call.
        unsafe {
            FindWindowExW(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                class.as_ptr(),
                std::ptr::null(),
            )
        }
    };

    if !workerw.is_null() {
        Some(workerw)
    } else {
        Some(progman)
    }
}

#[cfg(target_os = "windows")]
fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(target_os = "windows")]
fn initial_window_pos(size: egui::Vec2) -> egui::Pos2 {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    // SAFETY: GetSystemMetrics is thread-safe and requires no pointers.
    let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) as f32 };
    let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) as f32 };
    let margin = 16.0_f32;

    clamp_window_position(
        egui::pos2(
            (screen_width - size.x - margin).max(0.0),
            (screen_height - size.y - margin).max(0.0),
        ),
        size,
    )
}

#[cfg(not(target_os = "windows"))]
fn initial_window_pos(_size: egui::Vec2) -> egui::Pos2 {
    egui::pos2(16.0, 16.0)
}

#[cfg(target_os = "windows")]
fn platform_visible_rect() -> Option<egui::Rect> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    };

    // SAFETY: GetSystemMetrics is thread-safe and requires no pointers.
    let x = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) as f32 };
    // SAFETY: GetSystemMetrics is thread-safe and requires no pointers.
    let y = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) as f32 };
    // SAFETY: GetSystemMetrics is thread-safe and requires no pointers.
    let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) as f32 };
    // SAFETY: GetSystemMetrics is thread-safe and requires no pointers.
    let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) as f32 };

    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    Some(egui::Rect::from_min_size(
        egui::pos2(x, y),
        egui::vec2(width, height),
    ))
}

#[cfg(not(target_os = "windows"))]
fn platform_visible_rect() -> Option<egui::Rect> {
    None
}

fn clamp_window_position(pos: egui::Pos2, window_size: egui::Vec2) -> egui::Pos2 {
    let Some(bounds) = platform_visible_rect() else {
        return pos;
    };

    let min_x = bounds.min.x;
    let min_y = bounds.min.y;
    let max_x = (bounds.max.x - window_size.x).max(min_x);
    let max_y = (bounds.max.y - window_size.y).max(min_y);

    egui::pos2(pos.x.clamp(min_x, max_x), pos.y.clamp(min_y, max_y))
}

fn position_changed(last: egui::Pos2, current: egui::Pos2) -> bool {
    (last.x - current.x).abs() > 0.5 || (last.y - current.y).abs() > 0.5
}

fn window_position_file() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(app_data) = std::env::var_os("APPDATA") {
            return PathBuf::from(app_data)
                .join("BlinkSpark")
                .join("window-position.txt");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(config_home)
                .join("blinkspark")
                .join("window-position.txt");
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join(".config")
                .join("blinkspark")
                .join("window-position.txt");
        }
    }

    PathBuf::from(".blinkspark-window-position.txt")
}

fn desktop_pin_file() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(app_data) = std::env::var_os("APPDATA") {
            return PathBuf::from(app_data)
                .join("BlinkSpark")
                .join("desktop-pin.txt");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(config_home)
                .join("blinkspark")
                .join("desktop-pin.txt");
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join(".config")
                .join("blinkspark")
                .join("desktop-pin.txt");
        }
    }

    PathBuf::from(".blinkspark-desktop-pin.txt")
}

fn load_saved_window_position() -> Option<egui::Pos2> {
    let path = window_position_file();
    let raw = fs::read_to_string(path).ok()?;
    let raw = raw.trim();
    let mut parts = raw.split(',');
    let x = parts.next()?.trim().parse::<f32>().ok()?;
    let y = parts.next()?.trim().parse::<f32>().ok()?;

    if x.is_finite() && y.is_finite() {
        Some(egui::pos2(x, y))
    } else {
        None
    }
}

fn save_window_position(path: &PathBuf, pos: egui::Pos2) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{:.2},{:.2}\n", pos.x, pos.y))
}

fn load_saved_desktop_pin() -> bool {
    let path = desktop_pin_file();
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return false,
    };

    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn save_desktop_pin(path: &PathBuf, pin_to_desktop: bool) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let serialized = if pin_to_desktop { "1\n" } else { "0\n" };
    fs::write(path, serialized)
}

fn load_app_icon() -> Option<egui::IconData> {
    eframe::icon_data::from_png_bytes(include_bytes!(
        "../assets/branding/generated/blinkspark-icon-256.png"
    ))
    .ok()
}

#[cfg(target_os = "windows")]
fn load_platform_cjk_font_data() -> Option<egui::FontData> {
    let candidates = [
        (r"C:\Windows\Fonts\NotoSansSC-VF.ttf", 0_u32),
        (r"C:\Windows\Fonts\msyh.ttc", 0_u32),
        (r"C:\Windows\Fonts\msyh.ttc", 1_u32),
        (r"C:\Windows\Fonts\simhei.ttf", 0_u32),
        (r"C:\Windows\Fonts\simsun.ttc", 0_u32),
        (r"C:\Windows\Fonts\simsunb.ttf", 0_u32),
    ];

    for (path, index) in candidates {
        if let Ok(font_bytes) = fs::read(path) {
            let mut font_data = egui::FontData::from_owned(font_bytes);
            font_data.index = index;
            return Some(font_data);
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn load_platform_cjk_font_data() -> Option<egui::FontData> {
    let candidates = [
        ("/System/Library/Fonts/Hiragino Sans GB.ttc", 0_u32),
        ("/System/Library/Fonts/STHeiti Light.ttc", 0_u32),
        ("/System/Library/Fonts/STHeiti Medium.ttc", 0_u32),
        ("/System/Library/Fonts/Supplemental/Songti.ttc", 0_u32),
        ("/System/Library/Fonts/CJKSymbolsFallback.ttc", 0_u32),
        ("/Library/Fonts/Arial Unicode.ttf", 0_u32),
    ];

    for (path, index) in candidates {
        if let Ok(font_bytes) = fs::read(path) {
            let mut font_data = egui::FontData::from_owned(font_bytes);
            font_data.index = index;
            return Some(font_data);
        }
    }

    None
}

#[cfg(target_os = "linux")]
fn load_platform_cjk_font_data() -> Option<egui::FontData> {
    let candidates = [
        (
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            0_u32,
        ),
        (
            "/usr/share/fonts/opentype/noto/NotoSansCJKSC-Regular.otf",
            0_u32,
        ),
        (
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            0_u32,
        ),
        ("/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc", 0_u32),
    ];

    for (path, index) in candidates {
        if let Ok(font_bytes) = fs::read(path) {
            let mut font_data = egui::FontData::from_owned(font_bytes);
            font_data.index = index;
            return Some(font_data);
        }
    }

    None
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn load_platform_cjk_font_data() -> Option<egui::FontData> {
    None
}

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    if let Some(font_data) = load_platform_cjk_font_data() {
        fonts
            .font_data
            .insert("cjk".to_string(), std::sync::Arc::new(font_data));

        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, "cjk".to_string());
        }
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            family.push("cjk".to_string());
        }
    }

    ctx.set_fonts(fonts);
}

fn configure_visuals(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(SPACE_SM, SPACE_SM);
    style.spacing.button_padding = egui::vec2(14.0, 7.0);

    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = egui::Color32::TRANSPARENT;
    visuals.widgets.noninteractive.bg_fill = rgba(COLOR_BACKGROUND, 180);
    visuals.widgets.noninteractive.fg_stroke.color = rgb(COLOR_TITLE);
    visuals.widgets.inactive.bg_fill = rgba((255, 255, 255), 160);
    visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 13));
    visuals.widgets.inactive.fg_stroke.color = rgb(COLOR_TITLE);
    visuals.widgets.hovered.bg_fill = rgba(COLOR_PRIMARY, 46);
    visuals.widgets.hovered.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 20));
    visuals.widgets.active.bg_fill = rgba(COLOR_PRIMARY, 66);
    visuals.widgets.active.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 24));
    visuals.selection.bg_fill = rgba(COLOR_PRIMARY, 92);
    visuals.selection.stroke = egui::Stroke::new(1.0, rgb(COLOR_PRIMARY_GRADIENT_END));
    style.visuals = visuals;

    ctx.set_style(style);
}

fn main() -> eframe::Result {
    let args = Args::parse();

    if args.interval == 0 {
        eprintln!("interval must be greater than 0 minutes");
        std::process::exit(2);
    }

    let saved_position =
        load_saved_window_position().map(|pos| clamp_window_position(pos, WINDOW_SIZE));
    let saved_pin_to_desktop = load_saved_desktop_pin();

    #[cfg(target_os = "windows")]
    {
        start_windows_desktop_pin_guard();
        set_windows_desktop_pin_enabled(saved_pin_to_desktop);
    }

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("BlinkSpark")
        .with_inner_size(WINDOW_SIZE)
        .with_min_inner_size(WINDOW_MIN_SIZE)
        .with_position(saved_position.unwrap_or_else(|| initial_window_pos(WINDOW_SIZE)))
        .with_resizable(true)
        .with_transparent(true);

    #[cfg(not(target_os = "windows"))]
    if saved_pin_to_desktop {
        viewport = viewport.with_window_level(egui::WindowLevel::AlwaysOnBottom);
    }

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "BlinkSpark",
        native_options,
        Box::new(move |cc| {
            configure_fonts(&cc.egui_ctx);
            configure_visuals(&cc.egui_ctx);
            Ok(Box::new(CountdownApp::new(
                args,
                saved_position,
                saved_pin_to_desktop,
            )))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_countdown_uses_minutes_and_seconds_for_short_duration() {
        assert_eq!(format_countdown(Duration::from_secs(65)), "01:05");
    }

    #[test]
    fn format_countdown_includes_hours_for_long_duration() {
        assert_eq!(format_countdown(Duration::from_secs(3661)), "01:01:01");
    }

    #[test]
    fn countdown_progress_clamps_between_zero_and_one() {
        assert_eq!(
            countdown_progress(Duration::from_secs(60), Duration::from_secs(60)),
            1.0
        );
        assert_eq!(
            countdown_progress(Duration::from_secs(60), Duration::from_secs(0)),
            0.0
        );
        assert_eq!(
            countdown_progress(Duration::from_secs(60), Duration::from_secs(90)),
            1.0
        );
    }

    #[test]
    fn countdown_progress_keeps_fraction_near_edges() {
        let almost_full = countdown_progress(Duration::from_secs(1200), Duration::from_secs(1195));
        let almost_empty = countdown_progress(Duration::from_secs(1200), Duration::from_secs(5));

        assert!((almost_full - (1195.0 / 1200.0)).abs() < 1e-6);
        assert!((almost_empty - (5.0 / 1200.0)).abs() < 1e-6);
    }

    #[test]
    fn footer_spacer_never_negative() {
        assert_eq!(footer_spacer(60.0, 52.0), 8.0);
        assert_eq!(footer_spacer(20.0, 52.0), 0.0);
    }

    #[test]
    fn reset_timer_restores_progress_to_full() {
        let args = Args {
            lang: Lang::Zh,
            interval: 20,
            once: false,
            repeat: false,
        };
        let mut app = CountdownApp::new(args, None, false);
        app.displayed_progress = 0.37;

        app.reset_timer();

        assert_eq!(app.displayed_progress, 1.0);
    }

    #[test]
    fn rolling_deadline_forward_restores_progress_to_full() {
        let args = Args {
            lang: Lang::En,
            interval: 20,
            once: false,
            repeat: false,
        };
        let mut app = CountdownApp::new(args, None, false);
        let now = Instant::now();
        app.next_deadline = now - Duration::from_secs(1);
        app.displayed_progress = 0.04;

        app.roll_deadline_forward(now);

        assert_eq!(app.displayed_progress, 1.0);
        assert!(app.next_deadline > now);
    }
}
