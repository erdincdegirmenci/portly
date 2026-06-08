mod ports;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

// Windows'ta admin değilse yeniden başlat
#[cfg(windows)]
fn ensure_admin() {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let is_admin = std::process::Command::new("cmd")
        .args(["/c", "net session >nul 2>&1"])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !is_admin {
        let exe = std::env::current_exe().unwrap_or_default();
        let _ = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-WindowStyle",
                "Hidden",
                "-Command",
                &format!(
                    "Start-Process -FilePath '{}' -Verb RunAs -WindowStyle Hidden",
                    exe.display()
                ),
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
        std::process::exit(0);
    }
}

#[tauri::command]
async fn get_ports() -> Vec<ports::PortInfo> {
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        tokio::task::spawn_blocking(|| ports::scan_ports()),
    )
    .await;
    match result {
        Ok(Ok(p)) => p,
        _ => vec![],
    }
}

#[tauri::command]
async fn kill_port(pid: u32) -> bool {
    tokio::task::spawn_blocking(move || {
        #[cfg(windows)]
        {
            std::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
        #[cfg(unix)]
        {
            std::process::Command::new("kill")
                .args(["-9", &pid.to_string()])
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
    })
    .await
    .unwrap_or(false)
}

#[tauri::command]
async fn open_terminal(dir: String) -> bool {
    tokio::task::spawn_blocking(move || {
        #[cfg(windows)]
        {
            let target = if dir.is_empty() {
                std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string())
            } else {
                dir.clone()
            };
            // Windows Terminal varsa kullan, yoksa cmd
            let wt = std::process::Command::new("wt")
                .args(["--startingDirectory", &target])
                .spawn();
            if wt.is_err() {
                let _ = std::process::Command::new("cmd")
                    .args(["/c", "start", "cmd", "/K", &format!("cd /d \"{}\"", target)])
                    .spawn();
            }
            true
        }
        #[cfg(target_os = "macos")]
        {
            let target = if dir.is_empty() {
                std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
            } else { dir };
            let script = format!(
                "tell application \"Terminal\"\nactivate\ndo script \"cd '{}'\"\nend tell",
                target.replace('\'', "\\'")
            );
            std::process::Command::new("osascript")
                .args(["-e", &script])
                .spawn()
                .is_ok()
        }
    })
    .await
    .unwrap_or(false)
}

#[tauri::command]
async fn open_folder(dir: String) -> bool {
    tokio::task::spawn_blocking(move || {
        #[cfg(windows)]
        {
            let target = if dir.is_empty() {
                std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string())
            } else {
                dir.clone()
            };
            let _ = std::process::Command::new("explorer").arg(&target).spawn();
            true
        }
        #[cfg(target_os = "macos")]
        {
            let target = if dir.is_empty() {
                std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
            } else { dir };
            std::process::Command::new("open").arg(&target).spawn().is_ok()
        }
    })
    .await
    .unwrap_or(false)
}

#[tauri::command]
async fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

fn position_bottom_right<R: Runtime>(window: &tauri::WebviewWindow<R>) {
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let screen = monitor.size();
        let scale = monitor.scale_factor();
        let win_w = 360.0_f64;
        let win_h = 480.0_f64;
        let margin = 12.0_f64;
        let taskbar = 52.0_f64;

        let x = (screen.width  as f64 - (win_w + margin) * scale) as i32;
        let y = (screen.height as f64 - (win_h + taskbar + margin) * scale) as i32;

        let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
    }
}

fn toggle_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            position_bottom_right(&window);
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(windows)]
    ensure_admin();

    tauri::Builder::default()
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
            let quit = MenuItem::with_id(app, "quit", "Quit portly", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show / Hide", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("portly")
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_window(tray.app_handle());
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => toggle_window(app),
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_ports,
            kill_port,
            open_terminal,
            open_folder,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running portly");
}