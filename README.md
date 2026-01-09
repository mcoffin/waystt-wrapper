# waystt-wrapper

A GTK4 overlay wrapper for [waystt](https://github.com/mcoffin/waystt) that provides a visual recording indicator on Wayland compositors.

## What it does

1. Spawns `waystt --pipe-to wl-copy` (or a custom command)
2. Displays a microphone icon overlay using wlr-layer-shell
3. When you press **Escape**, sends `SIGUSR1` to waystt to stop recording
4. Exits with the same exit code as waystt

## Requirements

- Wayland compositor with layer-shell support (Sway, Hyprland, etc.)
- `gtk4-layer-shell` library
- `waystt` installed and in PATH
- `wl-copy` (from wl-clipboard) for the default command

## Building

```bash
cargo build --release
```

The binary will be at `target/release/waystt-wrapper`.

## Usage

```bash
waystt-wrapper [OPTIONS] [-- COMMAND...]
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--icon <NAME>` | `audio-input-microphone-symbolic` | Icon name from system theme |
| `--icon-size <PX>` | `48` | Icon size in pixels |
| `--position <POS>` | `top-right` | Overlay position: `top-left`, `top-right`, `bottom-left`, `bottom-right`, `center` |
| `--margin <PX>` | `20` | Margin from screen edges |

### Examples

Basic usage (uses `waystt --pipe-to wl-copy`):
```bash
waystt-wrapper
```

Custom position:
```bash
waystt-wrapper --position center --margin 0
```

Custom command:
```bash
waystt-wrapper -- waystt --pipe-to "cat >> ~/notes.txt"
```

### Sway configuration

```
bindsym $mod+r exec waystt-wrapper
```

## Environment

Set `RUST_LOG` for debug output:
```bash
RUST_LOG=waystt_wrapper=debug waystt-wrapper
```

## License

This project is licensed under the [GPL-3.0-or-later](LICENSE).
