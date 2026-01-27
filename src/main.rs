use std::time::Duration;

use clap::{Parser, ValueEnum};

mod notifier;

#[derive(Parser, Debug)]
#[command(name = "blinkspark", version, about = "20-20-20 blink reminder")]
struct Args {
    /// Reminder language: zh or en
    #[arg(long, value_enum, default_value_t = Lang::Zh)]
    lang: Lang,

    /// Interval in minutes before sending the reminder
    #[arg(long, default_value_t = 20)]
    interval: u64,

    /// Repeat reminders every interval until interrupted
    #[arg(long, default_value_t = false)]
    repeat: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
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
}

fn main() {
    let args = Args::parse();

    if args.interval == 0 {
        eprintln!("interval must be greater than 0 minutes");
        std::process::exit(2);
    }

    let interval = Duration::from_secs(args.interval.saturating_mul(60));
    let (title, body) = args.lang.message();

    if args.repeat {
        loop {
            std::thread::sleep(interval);
            if let Err(err) = notifier::notify(title, body) {
                eprintln!("failed to send notification: {err}");
            }
        }
    } else {
        std::thread::sleep(interval);
        if let Err(err) = notifier::notify(title, body) {
            eprintln!("failed to send notification: {err}");
            std::process::exit(1);
        }
    }
}
