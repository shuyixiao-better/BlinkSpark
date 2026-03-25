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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.lang.window_title());
            ui.separator();

            ui.label(self.lang.countdown_label());
            ui.heading(format_countdown(remaining));

            ui.separator();
            ui.label(self.lang.mode_label(self.once_mode));
            ui.label(self.lang.drag_hint());

            ui.horizontal(|ui| {
                ui.label(self.lang.language_label());
                if ui
                    .selectable_label(self.lang == Lang::En, "English")
                    .clicked()
                {
                    self.lang = Lang::En;
                }
                if ui
                    .selectable_label(self.lang == Lang::Zh, "中文")
                    .clicked()
                {
                    self.lang = Lang::Zh;
                }
            });

            if let Some(err) = &self.last_error {
                ui.separator();
                ui.colored_label(
                    egui::Color32::from_rgb(220, 80, 80),
                    format!("{} {}", self.lang.notify_error_prefix(), err),
                );
            }

            if let Some(err) = &self.last_position_error {
                ui.separator();
                ui.colored_label(
                    egui::Color32::from_rgb(220, 80, 80),
                    format!("{} {}", self.lang.position_error_prefix(), err),
                );
            }
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

fn main() -> eframe::Result {
    let args = Args::parse();

    if args.interval == 0 {
        eprintln!("interval must be greater than 0 minutes");
        std::process::exit(2);
    }

    let window_size = egui::vec2(320.0, 190.0);
    let saved_position = load_saved_window_position();
    let viewport = egui::ViewportBuilder::default()
        .with_title("BlinkSpark")
        .with_inner_size(window_size)
        .with_min_inner_size(window_size)
        .with_position(saved_position.unwrap_or_else(|| initial_window_pos(window_size)))
        .with_resizable(false);

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "BlinkSpark",
        native_options,
        Box::new(move |_cc| Ok(Box::new(CountdownApp::new(args, saved_position)))),
    )
}
