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
    use std::sync::OnceLock;

    const APP_BUNDLE_ID: &str = "io.github.shuyixiao.blinkspark";
    static APP_BIND_RESULT: OnceLock<Result<(), String>> = OnceLock::new();

    fn ensure_notification_application() -> Result<(), String> {
        APP_BIND_RESULT
            .get_or_init(|| {
                mac_notification_sys::set_application(APP_BUNDLE_ID).map_err(|err| err.to_string())
            })
            .clone()
    }

    pub fn notify(title: &str, body: &str) -> Result<(), String> {
        ensure_notification_application()?;
        mac_notification_sys::send_notification(title, None, body, None)
            .map(|_| ())
            .map_err(|err| err.to_string())
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::ptr;

    use windows_sys::Win32::UI::WindowsAndMessaging::{
        MessageBoxW, MB_ICONINFORMATION, MB_OK, MB_TOPMOST,
    };
    use winrt_notification::{Duration, Toast};

    fn to_wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn notify_with_message_box(title: &str, body: &str) -> Result<(), String> {
        let wide_title = to_wide(title);
        let wide_body = to_wide(body);

        // SAFETY: Both strings are NUL-terminated UTF-16 buffers that stay alive for the duration of the call.
        let result = unsafe {
            MessageBoxW(
                ptr::null_mut(),
                wide_body.as_ptr(),
                wide_title.as_ptr(),
                MB_OK | MB_ICONINFORMATION | MB_TOPMOST,
            )
        };

        if result == 0 {
            Err("MessageBoxW failed".to_string())
        } else {
            Ok(())
        }
    }

    pub fn notify(title: &str, body: &str) -> Result<(), String> {
        match Toast::new(Toast::POWERSHELL_APP_ID)
            .title(title)
            .text1(body)
            .duration(Duration::Short)
            .show()
        {
            Ok(()) => Ok(()),
            Err(toast_err) => notify_with_message_box(title, body).map_err(|message_box_err| {
                format!(
                    "toast failed: {}; fallback MessageBox failed: {}",
                    toast_err, message_box_err
                )
            }),
        }
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
