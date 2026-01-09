mod config;
mod error;
mod overlay;
mod process;

use std::cell::{Cell, RefCell};
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Duration;

use clap::Parser;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Application, EventControllerKey};
use tracing::{error, info, warn};

use config::{Args, Config};
use overlay::create_overlay_window;
use process::ChildProcess;

fn main() -> ExitCode {
    // Initialize tracing with RUST_LOG support
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("waystt_wrapper=info".parse().unwrap()),
        )
        .init();

    let args = Args::parse();
    let config = Config::from(args);

    info!("Starting waystt-wrapper");

    // Create GTK application
    let app = Application::builder()
        .application_id("com.github.mcoffin.waystt-wrapper")
        .build();

    // Store exit code for returning after GTK loop ends
    let exit_code: Rc<Cell<i32>> = Rc::new(Cell::new(0));
    let exit_code_for_activate = exit_code.clone();

    // Move config into the closure
    let config = Rc::new(config);

    app.connect_activate(move |app| {
        let exit_code = exit_code_for_activate.clone();
        let config = config.clone();

        // Spawn waystt child process
        let child = match ChildProcess::spawn(&config.command) {
            Ok(c) => c,
            Err(e) => {
                error!(error = %e, "Failed to spawn child process");
                exit_code.set(1);
                return;
            }
        };

        // Create overlay window
        let window = match create_overlay_window(app, &config) {
            Ok(w) => w,
            Err(e) => {
                error!(error = %e, "Failed to create overlay window");
                exit_code.set(1);
                return;
            }
        };

        // Wrap child in RefCell for interior mutability
        let child_cell: Rc<RefCell<Option<ChildProcess>>> = Rc::new(RefCell::new(Some(child)));

        // Setup keyboard controller for Escape key
        let controller = EventControllerKey::new();
        let child_for_key = child_cell.clone();
        let exit_code_for_key = exit_code.clone();
        let window_weak = window.downgrade();

        controller.connect_key_pressed(move |_, keyval, _, _| {
            if keyval == gdk::Key::Escape {
                info!("Escape pressed, initiating shutdown");

                if let Some(child) = child_for_key.borrow_mut().take() {
                    // Send SIGUSR1 to child
                    if let Err(e) = child.send_sigusr1() {
                        warn!(error = %e, "Failed to send SIGUSR1");
                    }

                    let exit_code_inner = exit_code_for_key.clone();
                    let window_weak_inner = window_weak.clone();

                    // Wait for child asynchronously
                    glib::spawn_future_local(async move {
                        match child.wait() {
                            Ok(status) => {
                                let code = status.code().unwrap_or(1);
                                info!(exit_code = code, "Child process exited");
                                exit_code_inner.set(code);
                            }
                            Err(e) => {
                                error!(error = %e, "Failed waiting for child");
                                exit_code_inner.set(1);
                            }
                        }

                        // Close window after child exits
                        if let Some(window) = window_weak_inner.upgrade() {
                            window.close();
                        }
                    });
                }

                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });

        window.add_controller(controller);

        // Handle window close (e.g., compositor closes it)
        let child_for_close = child_cell.clone();
        let exit_code_for_close = exit_code.clone();

        window.connect_close_request(move |_| {
            if let Some(mut child) = child_for_close.borrow_mut().take() {
                warn!("Window closed, killing child process");
                if let Err(e) = child.send_sigusr1() {
                    warn!(error = %e, "Failed to send SIGUSR1, force killing");
                    child.force_kill();
                }
                // Set exit code to indicate abnormal close
                exit_code_for_close.set(130); // Similar to Ctrl+C
            }
            glib::Propagation::Proceed
        });

        // Monitor child process for unexpected exit
        let child_for_monitor = child_cell.clone();
        let exit_code_for_monitor = exit_code.clone();
        let window_weak_monitor = window.downgrade();

        glib::timeout_add_local(Duration::from_millis(100), move || {
            if let Some(ref mut child) = *child_for_monitor.borrow_mut() {
                if let Ok(Some(status)) = child.try_wait() {
                    // Child already exited unexpectedly
                    let code = status.code().unwrap_or(1);
                    warn!(exit_code = code, "Child process exited unexpectedly");
                    exit_code_for_monitor.set(code);
                    if let Some(window) = window_weak_monitor.upgrade() {
                        window.close();
                    }
                    return glib::ControlFlow::Break;
                }
            } else {
                // Child was already taken (shutdown in progress)
                return glib::ControlFlow::Break;
            }
            glib::ControlFlow::Continue
        });

        window.present();
        info!("Overlay window presented, waiting for Escape key");
    });

    // Run GTK main loop (don't pass CLI args to GTK)
    let _status = app.run_with_args::<&str>(&[]);

    let code = exit_code.get();
    info!(exit_code = code, "waystt-wrapper exiting");

    ExitCode::from(code as u8)
}
