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

[ui.colors]
title = ["cyan"]
source = ["magenta"]
playlist = ["green"]
playlist_active = ["red"]
progress_bar = ["blue", "white"]
status = ["red"]
default = ["white"]
```

`sources` accepts directories, supported audio files, M3U/M3U8 playlists,
`file://` URIs, and HTTP(S) streams. A leading `~/` is expanded in both the
configuration and playlists. Directory scanning is recursive.

The default behavior shuffles the combined library. Set `no_shuffle = true` or
pass `--no-shuffle` to retain directory and playlist order.

Command-line sources replace configured sources for one run. `--no-config`
ignores configuration files, while `--config PATH` selects an explicit file.
`--write-config` writes the effective settings to the XDG location.
