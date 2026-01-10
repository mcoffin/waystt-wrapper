mod config;
mod overlay;
mod process;

use std::cell::{Cell, RefCell};
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Duration;

use clap::Parser;
use gtk4::gdk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, EventControllerKey, Image};
use tracing::*;

use config::{Args, Config};
use overlay::create_overlay_window;
use process::{killall, ChildProcess};

/// Shared state for the application's activate handler
struct AppState {
    exit_code: Rc<Cell<i32>>,
    config: Rc<Config>,
}

/// Wait for child process exit and update state accordingly
fn wait_for_child_exit(
    child: ChildProcess,
    exit_code: Rc<Cell<i32>>,
    window_weak: glib::WeakRef<ApplicationWindow>,
) {
    glib::spawn_future_local(async move {
        let result = gio::spawn_blocking(move || child.wait()).await;
        let code = match result {
            Ok(Ok(status)) => {
                let code = status.code().unwrap_or(1);
                info!(exit_code = code, "Child process exited");
                code
            }
            Ok(Err(e)) => {
                error!(error = %e, "Failed waiting for child");
                1
            }
            Err(e) => {
                error!(error = ?e, "spawn_blocking failed");
                1
            }
        };
        exit_code.set(code);

        if let Some(window) = window_weak.upgrade() {
            window.close();
        }
    });
}

/// Handle graceful shutdown initiated by Escape key
fn initiate_shutdown(
    child: ChildProcess,
    icon: &Image,
    exit_code: Rc<Cell<i32>>,
    window_weak: glib::WeakRef<ApplicationWindow>,
) {
    if let Err(e) = child.send_sigusr1() {
        warn!(error = %e, "Failed to send SIGUSR1");
    }

    icon.set_icon_name(Some("content-loading-symbolic"));
    wait_for_child_exit(child, exit_code, window_weak);
}

/// Handle the Escape key press event
fn handle_escape_press(
    m_state: gdk::ModifierType,
    child_cell: &Rc<RefCell<Option<ChildProcess>>>,
    icon: &Image,
    exit_code: Rc<Cell<i32>>,
    window_weak: glib::WeakRef<ApplicationWindow>,
) {
    info!("Escape pressed, initiating shutdown");

    let is_panic_combo =
        m_state.contains(gdk::ModifierType::ALT_MASK | gdk::ModifierType::CONTROL_MASK);
    if is_panic_combo {
        warn!("user pressed the panic exit hotkey, closing all windows");
        if let Err(e) = killall(env!("CARGO_PKG_NAME"), Some("-1")) {
            error!("error killing other windows, some may still exist: {e}");
        }
    }

    if let Some(child) = child_cell.borrow_mut().take() {
        initiate_shutdown(child, icon, exit_code, window_weak);
    }
}

/// Setup keyboard controller for Escape key handling
fn setup_key_controller(
    window: &ApplicationWindow,
    child_cell: Rc<RefCell<Option<ChildProcess>>>,
    icon: Rc<Image>,
    exit_code: Rc<Cell<i32>>,
) {
    let controller = EventControllerKey::new();
    let window_weak = window.downgrade();

    controller.connect_key_pressed(move |_, keyval, _, m_state| {
        if keyval != gdk::Key::Escape {
            return glib::Propagation::Proceed;
        }

        handle_escape_press(
            m_state,
            &child_cell,
            &icon,
            exit_code.clone(),
            window_weak.clone(),
        );
        glib::Propagation::Stop
    });

    window.add_controller(controller);
}

/// Handle window close request (e.g., compositor closes it)
fn setup_close_handler(
    window: &ApplicationWindow,
    child_cell: Rc<RefCell<Option<ChildProcess>>>,
    exit_code: Rc<Cell<i32>>,
) {
    window.connect_close_request(move |_| {
        if let Some(mut child) = child_cell.borrow_mut().take() {
            warn!("Window closed, killing child process");
            if let Err(e) = child.send_sigusr1() {
                warn!(error = %e, "Failed to send SIGUSR1, force killing");
                child.force_kill();
            }
            exit_code.set(130); // Similar to Ctrl+C
        }
        glib::Propagation::Proceed
    });
}

/// Handle unexpected child exit during monitoring
fn handle_unexpected_exit(
    status: std::process::ExitStatus,
    exit_code: &Rc<Cell<i32>>,
    window_weak: &glib::WeakRef<ApplicationWindow>,
) {
    let code = status.code().unwrap_or(1);
    warn!(exit_code = code, "Child process exited unexpectedly");
    exit_code.set(code);
    if let Some(window) = window_weak.upgrade() {
        window.close();
    }
}

/// Monitor child process for unexpected exit
fn setup_child_monitor(
    window: &ApplicationWindow,
    child_cell: Rc<RefCell<Option<ChildProcess>>>,
    exit_code: Rc<Cell<i32>>,
) {
    let window_weak = window.downgrade();

    glib::timeout_add_local(Duration::from_millis(100), move || {
        let mut child_ref = child_cell.borrow_mut();
        let Some(ref mut child) = *child_ref else {
            return glib::ControlFlow::Break;
        };

        match child.try_wait() {
            Ok(Some(status)) => {
                handle_unexpected_exit(status, &exit_code, &window_weak);
                glib::ControlFlow::Break
            }
            _ => glib::ControlFlow::Continue,
        }
    });
}

/// GTK application activate handler
fn on_activate(app: &Application, state: &AppState) {
    let child = match ChildProcess::spawn(&state.config.command) {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to spawn child process");
            state.exit_code.set(1);
            return;
        }
    };

    let (window, icon) = match create_overlay_window(app, &state.config) {
        Ok(w) => w,
        Err(e) => {
            error!(error = %e, "Failed to create overlay window");
            state.exit_code.set(1);
            return;
        }
    };

    let icon = Rc::new(icon);
    let child_cell: Rc<RefCell<Option<ChildProcess>>> = Rc::new(RefCell::new(Some(child)));

    setup_key_controller(&window, child_cell.clone(), icon.clone(), state.exit_code.clone());
    setup_close_handler(&window, child_cell.clone(), state.exit_code.clone());
    setup_child_monitor(&window, child_cell, state.exit_code.clone());

    window.present();
    info!("Overlay window presented, waiting for Escape key");
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("waystt_wrapper=info".parse().unwrap()),
        )
        .init();

    let args = Args::parse();
    let config = Config::from(args);

    info!("Starting waystt-wrapper");

    let app = Application::builder()
        .application_id("com.github.mcoffin.waystt-wrapper")
        .build();

    let state = AppState {
        exit_code: Rc::new(Cell::new(0)),
        config: Rc::new(config),
    };

    let exit_code = state.exit_code.clone();

    app.connect_activate(move |app| on_activate(app, &state));

    let _status = app.run_with_args::<&str>(&[]);

    let code = exit_code.get();
    info!(exit_code = code, "waystt-wrapper exiting");

    ExitCode::from(code as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let config = Config {
            icon: "test-icon".to_string(),
            icon_size: 64,
            position: config::Position::Center,
            margin: 10,
            command: vec!["echo".to_string()],
        };

        let state = AppState {
            exit_code: Rc::new(Cell::new(0)),
            config: Rc::new(config),
        };

        assert_eq!(state.exit_code.get(), 0);
        assert_eq!(state.config.icon, "test-icon");
    }



    #[test]
    fn test_panic_combo_detection() {
        // Test that Ctrl+Alt is detected correctly
        let modifiers = gdk::ModifierType::ALT_MASK | gdk::ModifierType::CONTROL_MASK;
        let is_panic =
            modifiers.contains(gdk::ModifierType::ALT_MASK | gdk::ModifierType::CONTROL_MASK);
        assert!(is_panic);
    }

    #[test]
    fn test_non_panic_combo() {
        // Test that Escape without modifiers doesn't trigger panic
        let modifiers = gdk::ModifierType::empty();
        let is_panic =
            modifiers.contains(gdk::ModifierType::ALT_MASK | gdk::ModifierType::CONTROL_MASK);
        assert!(!is_panic);
    }

    #[test]
    fn test_partial_modifier_not_panic() {
        // Test that Ctrl+Escape (without Alt) doesn't trigger panic
        let modifiers = gdk::ModifierType::CONTROL_MASK;
        let is_panic =
            modifiers.contains(gdk::ModifierType::ALT_MASK | gdk::ModifierType::CONTROL_MASK);
        assert!(!is_panic);
    }

    #[test]
    fn test_alt_only_not_panic() {
        // Test that Alt+Escape (without Ctrl) doesn't trigger panic
        let modifiers = gdk::ModifierType::ALT_MASK;
        let is_panic =
            modifiers.contains(gdk::ModifierType::ALT_MASK | gdk::ModifierType::CONTROL_MASK);
        assert!(!is_panic);
    }
}
