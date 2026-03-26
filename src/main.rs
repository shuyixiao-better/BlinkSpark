#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::{
    fs,
    io,
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::{Parser, ValueEnum};
use eframe::egui;

mod notifier;

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
                "该眨眼啦",
                "遵循 20-20-20 规则：看 6 米外 20 秒。",
            ),
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
    last_error: Option<String>,
    last_position_error: Option<String>,
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
            last_error: None,
            last_position_error: None,
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
}

impl eframe::App for CountdownApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
        ctx.request_repaint_after(Duration::from_millis(200));

        let remaining = self.next_deadline.saturating_duration_since(Instant::now());
        let progress = countdown_progress(self.interval, remaining);
        let accent = progress_accent_color(1.0 - progress);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(240, 245, 250))
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(self.lang.window_title())
                            .size(25.0)
                            .strong()
                            .color(egui::Color32::from_rgb(25, 52, 77)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(self.lang.mode_label(self.once_mode))
                                .size(13.0)
                                .color(egui::Color32::from_rgb(90, 107, 126)),
                        );
                    });
                });
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);

                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(252, 254, 255))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(221, 231, 240)))
                    .corner_radius(egui::CornerRadius::same(16))
                    .inner_margin(egui::Margin::same(14))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(self.lang.countdown_label())
                                .size(14.0)
                                .color(egui::Color32::from_rgb(104, 119, 136)),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(format_countdown(remaining))
                                .size(48.0)
                                .monospace()
                                .strong()
                                .color(egui::Color32::from_rgb(28, 44, 62)),
                        );
                        ui.add_space(8.0);
                        ui.add(
                            egui::ProgressBar::new(progress)
                                .desired_width(f32::INFINITY)
                                .fill(accent),
                        );
                    });

                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(self.lang.language_label())
                            .size(13.0)
                            .color(egui::Color32::from_rgb(97, 112, 129)),
                    );

                    if ui
                        .add(egui::Button::new("EN").selected(self.lang == Lang::En))
                        .clicked()
                    {
                        self.lang = Lang::En;
                    }
                    if ui
                        .add(egui::Button::new("ZH").selected(self.lang == Lang::Zh))
                        .clicked()
                    {
                        self.lang = Lang::Zh;
                    }
                });

                if let Some(err) = &self.last_error {
                    ui.add_space(6.0);
                    ui.colored_label(
                        egui::Color32::from_rgb(195, 56, 56),
                        format!("{} {}", self.lang.notify_error_prefix(), err),
                    );
                }

                if let Some(err) = &self.last_position_error {
                    ui.add_space(6.0);
                    ui.colored_label(
                        egui::Color32::from_rgb(195, 56, 56),
                        format!("{} {}", self.lang.position_error_prefix(), err),
                    );
                }

                let bottom_row_height = 34.0;
                ui.add_space((ui.available_height() - bottom_row_height).max(0.0));

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(self.lang.drag_hint())
                            .size(12.0)
                            .color(egui::Color32::from_rgb(117, 130, 146)),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_sized(
                                [132.0, 28.0],
                                egui::Button::new(self.lang.reset_button_label()),
                            )
                            .clicked()
                        {
                            self.reset_timer();
                        }
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

fn lerp_u8(from: u8, to: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    (from as f32 + (to as f32 - from as f32) * t).round() as u8
}

fn progress_accent_color(progress: f32) -> egui::Color32 {
    let t = progress.clamp(0.0, 1.0);
    egui::Color32::from_rgb(
        lerp_u8(33, 240, t),
        lerp_u8(133, 134, t),
        lerp_u8(94, 64, t),
    )
}

#[cfg(target_os = "windows")]
fn initial_window_pos(size: egui::Vec2) -> egui::Pos2 {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    // SAFETY: GetSystemMetrics is thread-safe and requires no pointers.
    let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) as f32 };
    let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) as f32 };
    let margin = 16.0_f32;

    egui::pos2(
        (screen_width - size.x - margin).max(0.0),
        (screen_height - size.y - margin).max(0.0),
    )
}

#[cfg(not(target_os = "windows"))]
fn initial_window_pos(_size: egui::Vec2) -> egui::Pos2 {
    egui::pos2(16.0, 16.0)
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
fn load_windows_cjk_font_bytes() -> Option<Vec<u8>> {
    let candidates = [
        r"C:\Windows\Fonts\NotoSansSC-VF.ttf",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simsunb.ttf",
    ];

    candidates.iter().find_map(|path| fs::read(path).ok())
}

#[cfg(not(target_os = "windows"))]
fn load_windows_cjk_font_bytes() -> Option<Vec<u8>> {
    None
}

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    if let Some(font_bytes) = load_windows_cjk_font_bytes() {
        fonts.font_data.insert(
            "cjk".to_string(),
            std::sync::Arc::new(egui::FontData::from_owned(font_bytes)),
        );

        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, "cjk".to_string());
        }
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            family.insert(0, "cjk".to_string());
        }
    }

    ctx.set_fonts(fonts);
}

fn configure_visuals(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);

    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = egui::Color32::from_rgb(240, 245, 250);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(240, 245, 250);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(232, 238, 245);
    visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(40, 64, 89);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(220, 233, 245);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(206, 224, 241);
    visuals.selection.bg_fill = egui::Color32::from_rgb(217, 229, 242);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(62, 95, 126));
    style.visuals = visuals;

    ctx.set_style(style);
}

fn main() -> eframe::Result {
    let args = Args::parse();

    if args.interval == 0 {
        eprintln!("interval must be greater than 0 minutes");
        std::process::exit(2);
    }

    let window_size = egui::vec2(420.0, 270.0);
    let saved_position = load_saved_window_position();
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("BlinkSpark")
        .with_inner_size(window_size)
        .with_min_inner_size(egui::vec2(380.0, 250.0))
        .with_position(saved_position.unwrap_or_else(|| initial_window_pos(window_size)))
        .with_resizable(true);

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
}
