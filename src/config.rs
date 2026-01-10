use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum Position {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    #[default]
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
    #[arg(long, default_value = "96", value_parser = clap::value_parser!(i32).range(1..))]
    pub icon_size: i32,

    /// Position of the overlay on screen
    #[arg(long, value_enum, default_value = "center")]
    pub position: Position,

    /// Margin from screen edges in pixels
    #[arg(long, default_value = "20", value_parser = clap::value_parser!(i32).range(0..))]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_args() {
        let args = Args::try_parse_from(&["waystt-wrapper"]).unwrap();
        assert_eq!(args.icon, "audio-input-microphone-symbolic");
        assert_eq!(args.icon_size, 96);
        assert_eq!(args.margin, 20);
        assert!(matches!(args.position, Position::Center));
        assert!(args.command.is_empty());
    }



    #[test]
    fn test_position_parsing() {
        // Test TopLeft
        let args = Args::try_parse_from(&["waystt-wrapper", "--position", "top-left"]).unwrap();
        assert!(matches!(args.position, Position::TopLeft));

        // Test TopRight
        let args = Args::try_parse_from(&["waystt-wrapper", "--position", "top-right"]).unwrap();
        assert!(matches!(args.position, Position::TopRight));

        // Test BottomLeft
        let args =
            Args::try_parse_from(&["waystt-wrapper", "--position", "bottom-left"]).unwrap();
        assert!(matches!(args.position, Position::BottomLeft));

        // Test BottomRight
        let args =
            Args::try_parse_from(&["waystt-wrapper", "--position", "bottom-right"]).unwrap();
        assert!(matches!(args.position, Position::BottomRight));

        // Test Center
        let args = Args::try_parse_from(&["waystt-wrapper", "--position", "center"]).unwrap();
        assert!(matches!(args.position, Position::Center));
    }

    #[test]
    fn test_config_default_command() {
        let args = Args::try_parse_from(&["waystt-wrapper"]).unwrap();
        let config = Config::from(args);

        assert_eq!(config.command.len(), 3);
        assert_eq!(config.command[0], "waystt");
        assert_eq!(config.command[1], "--pipe-to");
        assert_eq!(config.command[2], "wl-copy");
    }

    #[test]
    fn test_config_custom_command() {
        let args =
            Args::try_parse_from(&["waystt-wrapper", "--", "custom-cmd", "arg1", "arg2"])
                .unwrap();
        let config = Config::from(args);

        assert_eq!(config.command.len(), 3);
        assert_eq!(config.command[0], "custom-cmd");
        assert_eq!(config.command[1], "arg1");
        assert_eq!(config.command[2], "arg2");
    }

    #[test]
    fn test_icon_size_custom() {
        let args = Args::try_parse_from(&["waystt-wrapper", "--icon-size", "128"]).unwrap();
        assert_eq!(args.icon_size, 128);
    }

    #[test]
    fn test_margin_custom() {
        let args = Args::try_parse_from(&["waystt-wrapper", "--margin", "50"]).unwrap();
        assert_eq!(args.margin, 50);
    }

    #[test]
    fn test_icon_custom() {
        let args =
            Args::try_parse_from(&["waystt-wrapper", "--icon", "microphone-sensitivity-high"])
                .unwrap();
        assert_eq!(args.icon, "microphone-sensitivity-high");
    }

    #[test]
    fn test_all_args_combined() {
        let args = Args::try_parse_from(&[
            "waystt-wrapper",
            "--icon",
            "custom-icon",
            "--icon-size",
            "200",
            "--position",
            "top-left",
            "--margin",
            "30",
            "--",
            "echo",
            "test",
        ])
        .unwrap();

        assert_eq!(args.icon, "custom-icon");
        assert_eq!(args.icon_size, 200);
        assert!(matches!(args.position, Position::TopLeft));
        assert_eq!(args.margin, 30);
        assert_eq!(args.command, vec!["echo", "test"]);
    }

    #[test]
    fn test_invalid_position() {
        let result = Args::try_parse_from(&["waystt-wrapper", "--position", "invalid"]);
        assert!(result.is_err());
    }



    #[test]
    fn test_config_conversion_preserves_fields() {
        let args = Args {
            icon: "test-icon".to_string(),
            icon_size: 150,
            position: Position::BottomRight,
            margin: 40,
            command: vec!["test".to_string()],
        };

        let config = Config::from(args);

        assert_eq!(config.icon, "test-icon");
        assert_eq!(config.icon_size, 150);
        assert!(matches!(config.position, Position::BottomRight));
        assert_eq!(config.margin, 40);
        assert_eq!(config.command, vec!["test"]);
    }

    #[test]
    fn test_icon_size_validation_rejects_zero() {
        let result = Args::try_parse_from(&["waystt-wrapper", "--icon-size", "0"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_icon_size_validation_rejects_negative() {
        let result = Args::try_parse_from(&["waystt-wrapper", "--icon-size", "-1"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_margin_validation_rejects_negative() {
        let result = Args::try_parse_from(&["waystt-wrapper", "--margin", "-1"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_margin_validation_accepts_zero() {
        let args = Args::try_parse_from(&["waystt-wrapper", "--margin", "0"]).unwrap();
        assert_eq!(args.margin, 0);
    }
}
