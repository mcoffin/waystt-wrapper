use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum Position {
    TopLeft,
    #[default]
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

#[derive(Parser, Debug)]
#[command(name = "waystt-wrapper")]
#[command(about = "GTK4 overlay wrapper for waystt speech-to-text")]
#[command(version)]
pub struct Args {
    /// Icon name from the system theme
    #[arg(long, default_value = "audio-input-microphone-symbolic")]
    pub icon: String,

    /// Icon size in pixels
    #[arg(long, default_value = "48")]
    pub icon_size: i32,

    /// Position of the overlay on screen
    #[arg(long, value_enum, default_value = "top-right")]
    pub position: Position,

    /// Margin from screen edges in pixels
    #[arg(long, default_value = "20")]
    pub margin: i32,

    /// Command to execute (defaults to "waystt --pipe-to wl-copy")
    #[arg(trailing_var_arg = true, num_args = 0..)]
    pub command: Vec<String>,
}

#[derive(Debug)]
pub struct Config {
    pub icon: String,
    pub icon_size: i32,
    pub position: Position,
    pub margin: i32,
    pub command: Vec<String>,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        let command = if args.command.is_empty() {
            vec![
                "waystt".to_string(),
                "--pipe-to".to_string(),
                "wl-copy".to_string(),
            ]
        } else {
            args.command
        };

        Self {
            icon: args.icon,
            icon_size: args.icon_size,
            position: args.position,
            margin: args.margin,
            command,
        }
    }
}
