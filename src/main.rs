#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::{
    fs, io,
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::{Parser, ValueEnum};
use eframe::egui;

mod notifier;

const WINDOW_SIZE: egui::Vec2 = egui::vec2(420.0, 290.0);
const WINDOW_MIN_SIZE: egui::Vec2 = egui::vec2(380.0, 270.0);

const PANEL_PADDING: i8 = 20;
const CARD_PADDING: i8 = 18;
const CARD_RADIUS: u8 = 20;
const CONTROL_RADIUS: u8 = 14;
const CONTROL_HEIGHT: f32 = 30.0;
const RESET_BUTTON_WIDTH: f32 = 156.0;
const TOGGLE_ACTION_GAP: f32 = 24.0;
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
            Lang::Zh => ("该眨眼啦", "遵循 20-20-20 规则：看 6 米外 20 秒。"),
            Lang::En => (
                "Time to blink",
                "Follow the 20-20-20 rule: look 20 feet away for 20 seconds.",
            ),
        }
    }

    fn window_title(self) -> &'static str {
        match self {
            Lang::Zh => "眨眼提醒",
            Lang::En => "Blink Reminder",
        }
    }

    fn countdown_label(self) -> &'static str {
        match self {
            Lang::Zh => "下次提醒倒计时",
            Lang::En => "Next reminder in",
        }
    }

    fn mode_label(self, once_mode: bool) -> &'static str {
        match (self, once_mode) {
            (Lang::Zh, true) => "模式：单次提醒",
            (Lang::Zh, false) => "模式：循环提醒",
            (Lang::En, true) => "Mode: once",
            (Lang::En, false) => "Mode: repeat",
        }
    }

    fn drag_hint(self) -> &'static str {
        match self {
            Lang::Zh => "可拖动窗口到任意位置",
            Lang::En => "Drag this window to place it anywhere",
        }
    }

    fn language_label(self) -> &'static str {
        match self {
            Lang::Zh => "语言",
            Lang::En => "Language",
        }
    }

    #[allow(dead_code)]
    fn reset_button_label(self) -> &'static str {
        match self {
            Lang::Zh => "重置计时",
            Lang::En => "Reset timer",
        }
    }

    fn notify_error_prefix(self) -> &'static str {
        match self {
            Lang::Zh => "通知失败：",
            Lang::En => "Notification failed:",
        }
    }

    fn position_error_prefix(self) -> &'static str {
        match self {
            Lang::Zh => "保存窗口位置失败：",
            Lang::En => "Failed to save window position:",
        }
    }
}

struct CountdownApp {
    interval: Duration,
    next_deadline: Instant,
    lang: Lang,
    once_mode: bool,
    position_path: PathBuf,
    last_saved_position: Option<egui::Pos2>,
    last_position_save_at: Option<Instant>,
    last_visibility_recover_at: Option<Instant>,
    last_error: Option<String>,
    last_position_error: Option<String>,
    displayed_progress: f32,
    finished: bool,
}

impl CountdownApp {
    fn new(args: Args, initial_saved_position: Option<egui::Pos2>) -> Self {
        let interval = Duration::from_secs(args.interval.saturating_mul(60));
        Self {
            interval,
            next_deadline: Instant::now() + interval,
            lang: args.lang,
            once_mode: args.once && !args.repeat,
            position_path: window_position_file(),
            last_saved_position: initial_saved_position,
            last_position_save_at: None,
            last_visibility_recover_at: None,
            last_error: None,
            last_position_error: None,
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
            while self.next_deadline <= now {
                self.next_deadline += self.interval;
            }
        }
    }

    fn reset_timer(&mut self) {
        self.next_deadline = Instant::now() + self.interval;
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
                                    ui.add_space(TOGGLE_ACTION_GAP);
                                    if primary_reset_button(ui).clicked() {
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

fn primary_reset_button(ui: &mut egui::Ui) -> egui::Response {
    let corner = egui::CornerRadius::same((CONTROL_HEIGHT / 2.0).round() as u8);
    ui.scope(|ui| {
        let visuals = ui.visuals_mut();
        visuals.widgets.hovered.bg_fill = rgb(COLOR_PRIMARY_GRADIENT_END);
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, rgb(COLOR_PRIMARY_GRADIENT_END));
        visuals.widgets.active.bg_fill = rgb(COLOR_PRIMARY_GRADIENT_END);
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, rgb(COLOR_PRIMARY_GRADIENT_END));

        let label = egui::RichText::new("↻ 重置计时")
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
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("BlinkSpark")
        .with_inner_size(WINDOW_SIZE)
        .with_min_inner_size(WINDOW_MIN_SIZE)
        .with_position(saved_position.unwrap_or_else(|| initial_window_pos(WINDOW_SIZE)))
        .with_resizable(true)
        .with_transparent(true);

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
            Ok(Box::new(CountdownApp::new(args, saved_position)))
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
}
