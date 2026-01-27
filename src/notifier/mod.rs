#[cfg(target_os = "linux")]
mod platform {
    pub fn notify(title: &str, body: &str) -> Result<(), String> {
        notify_rust::Notification::new()
            .summary(title)
            .body(body)
            .show()
            .map(|_| ())
            .map_err(|err| err.to_string())
    }
}

#[cfg(target_os = "macos")]
mod platform {
    pub fn notify(title: &str, body: &str) -> Result<(), String> {
        mac_notification_sys::send_notification(title, None, Some(body), None, None)
            .map_err(|err| err.to_string())
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use winrt_notification::{Toast, ToastDuration};

    pub fn notify(title: &str, body: &str) -> Result<(), String> {
        Toast::new(Toast::POWERSHELL_APP_ID)
            .title(title)
            .text1(body)
            .duration(ToastDuration::Short)
            .show()
            .map_err(|err| err.to_string())
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod platform {
    pub fn notify(_title: &str, _body: &str) -> Result<(), String> {
        Err("unsupported platform for system notifications".to_string())
    }
}

pub fn notify(title: &str, body: &str) -> Result<(), String> {
    platform::notify(title, body)
}
