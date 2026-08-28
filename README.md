# CloudRuby

CloudRuby is a terminal music player. It plays audio from local
folders, individual files, M3U/M3U8 playlists, and HTTP(S) streams such as
internet-radio and Icecast stations. The interface is written with Ratatui and
playback uses GStreamer 1.x.

On wide terminals, the right pane shows a live, rotating radial FFT visualization.
## Requirements

- Rust 1.92 or newer
- GStreamer 1.x development files and playback plugins
- A UTF-8 terminal

On Debian or Ubuntu:

```sh
sudo apt install build-essential pkg-config libgstreamer1.0-dev \
  gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
  gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly
```

On Fedora:

```sh
sudo dnf install gcc pkgconf-pkg-config gstreamer1-devel \
  gstreamer1-plugins-base gstreamer1-plugins-good \
  gstreamer1-plugins-bad-free gstreamer1-plugins-ugly-free
```

On Arch Linux:

```sh
sudo pacman -S base-devel pkgconf gstreamer gst-plugins-base \
  gst-plugins-good gst-plugins-bad gst-plugins-ugly
```

## Build and run

```sh
cargo build --release
cargo install --path .
```

Play every supported audio file below a folder:

```sh
cloudruby ~/Music
```

Play a playlist in its declared order:

```sh
cloudruby --no-shuffle favorites.m3u8
```

Multiple inputs and direct streams can be combined:

```sh
cloudruby --no-shuffle ~/Music radio.m3u \
  https://radio.example.org/live.ogg
```

Sources can also be supplied one per line through standard input:

```sh
printf '%s\n' ~/Music/favorite.mp3 https://radio.example.org/live.ogg | cloudruby --no-shuffle
```

Directory scanning is recursive and recognizes AAC, FLAC, M4A, MP3, OGA, OGG,
Opus, WAV, and WebM files. M3U entries may be absolute paths, paths relative to
the playlist, `file://` URIs, or HTTP(S) URLs. `#EXTINF` titles are displayed when
present.

## Keyboard controls

| Key | Action |
| --- | --- |
| `n`, `N`, Down | Next entry |
| `p`, `P`, Up | Previous entry |
| Space | Pause or resume |
| `+`, `=` | Raise volume |
| `-`, `_` | Lower volume |
| `m`, `M` | Toggle mute |
| `v`, `V` | Toggle source information |
| `q`, `Q`, Esc | Quit |

## Configuration

CloudRuby reads `$XDG_CONFIG_HOME/cloudruby/config.toml`, normally
`~/.config/cloudruby/config.toml`:

```toml
sources = ["~/Music", "/home/me/playlists/radio.m3u8"]
no_shuffle = true

[ui.colors]
title = { fg = "#88c0d0", modifiers = ["bold"] }
playlist_active = { fg = "black", bg = "#88c0d0", modifiers = ["bold"] }
progress_fill = { fg = "#a3be8c", bg = "ansi:236" }
visualizer_low = { fg = "#2f81a8" }
visualizer_mid = { fg = "#88c0d0" }
visualizer_high = { fg = "#b48ead" }
```

Command-line sources replace configured sources for that run. See
[configuration](doc/configuration.md) and [colors](doc/colors.md) for details.

`--no-shuffle` also accepts the legacy forms `--no_shuffle` and
`--no-shuffle=true`.

## Screenshots



![Terminal: Konsole, Font: Hack 14pt](doc/anim_1.gif?raw=true "Figure 1")

![Terminal: Konsole, Font: Hack 14pt](doc/screenshot_1.png?raw=true "Figure 2")


## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## License

See [LICENSE](LICENSE).
