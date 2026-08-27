# Configuration

CloudRuby reads TOML from `$XDG_CONFIG_HOME/cloudruby/config.toml`, normally
`~/.config/cloudruby/config.toml`.

```toml
sources = [
  "~/Music",
  "/home/me/playlists/favorites.m3u8",
  "https://radio.example.org/live.ogg",
]
no_shuffle = true

[ui]
border_type = "rounded"
unicode = true
visualizer_sensitivity = 1.25

[ui.colors]
app = { fg = "#d8dee9", bg = "ansi:234" }
title = { fg = "#88c0d0", modifiers = ["bold"] }
playlist_active = { fg = "black", bg = "#88c0d0", modifiers = ["bold"] }
progress_fill = { fg = "#a3be8c", bg = "ansi:236", modifiers = ["bold"] }
```

`sources` accepts directories, supported audio files, M3U/M3U8 playlists,
`file://` URIs, and HTTP(S) streams. A leading `~/` is expanded in both the
configuration and playlists. Directory scanning is recursive.

The default behavior shuffles the combined library. Set `no_shuffle = true` or
pass `--no-shuffle` to retain directory and playlist order.

Command-line sources replace configured sources for one run. `--no-config`
ignores configuration files, while `--config PATH` selects an explicit file.
`--write-config` writes the effective settings to the XDG location.

The `[ui]` table also accepts `border_type = "rounded"`, `unicode = true`, and
`visualizer_sensitivity = 1.25`. Sensitivity values above `1.0` amplify the FFT
display; values below `1.0` make it calmer.
See [colors](colors.md) for all style keys, color formats, modifiers, and border
options.
