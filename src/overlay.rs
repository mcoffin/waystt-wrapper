use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider, Image};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use tracing::info;

use crate::config::{Config, Position};
use crate::error::{Result, WaysttWrapperError};

pub fn create_overlay_window(app: &Application, config: &Config) -> Result<(ApplicationWindow, Image)> {
    // Check layer shell support
    if !gtk4_layer_shell::is_supported() {
        return Err(WaysttWrapperError::LayerShellNotSupported);
    }

    info!("Creating overlay window");

    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(config.icon_size + 20)
        .default_height(config.icon_size + 20)
        .build();

    // Initialize layer shell BEFORE the window is realized
    window.init_layer_shell();

    // Set to overlay layer (on top of everything)
    window.set_layer(Layer::Overlay);

    // Set keyboard mode to exclusively capture keyboard input
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    // Set anchors based on position
    match config.position {
        Position::TopLeft => {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Left, true);
        }
        Position::TopRight => {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Right, true);
        }
        Position::BottomLeft => {
            window.set_anchor(Edge::Bottom, true);
            window.set_anchor(Edge::Left, true);
        }
        Position::BottomRight => {
            window.set_anchor(Edge::Bottom, true);
            window.set_anchor(Edge::Right, true);
        }
        Position::Center => {
            // No anchors = centered
        }
    }

    // Set margins from edge
    window.set_margin(Edge::Top, config.margin);
    window.set_margin(Edge::Bottom, config.margin);
    window.set_margin(Edge::Left, config.margin);
    window.set_margin(Edge::Right, config.margin);

    // Create and add the microphone icon
    let icon = Image::from_icon_name(&config.icon);
    icon.set_pixel_size(config.icon_size);
    window.set_child(Some(&icon));

    // Add CSS styling for visibility
    let provider = CssProvider::new();
    provider.load_from_data(
        "window {
            background-color: rgba(50, 50, 50, 0.8);
            border-radius: 10px;
            padding: 10px;
        }
        image {
            color: #ff5555;
        }",
    );

    gtk4::style_context_add_provider_for_display(
        &Display::default().expect("Could not get default display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    info!(position = ?config.position, "Overlay window created");

    Ok((window, icon))
}
