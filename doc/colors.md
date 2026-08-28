# Themes and colors

CloudRuby exposes semantic styles for every major interface element. A style can
set a foreground, background, and any number of terminal modifiers:

```toml
[ui]
border_type = "rounded"
unicode = true
visualizer_sensitivity = 1.25

[ui.colors]
app = { fg = "#d8dee9", bg = "ansi:234" }
border = { fg = "#4c566a" }
logo = { fg = "#88c0d0", modifiers = ["bold"] }
title = { fg = "#eceff4", modifiers = ["bold"] }
tabs = { fg = "dark_gray" }
tabs_active = { fg = "light_cyan", modifiers = ["bold", "underlined"] }
progress_track = { fg = "ansi:237", bg = "ansi:234" }
progress_fill = { fg = "#a3be8c", bg = "ansi:236", modifiers = ["bold"] }
visualizer_low = { fg = "#2f81a8" }
visualizer_mid = { fg = "#88c0d0" }
visualizer_high = { fg = "#b48ead" }
visualizer_peak = { fg = "#ebcb8b", modifiers = ["bold"] }
visualizer_hot = { fg = "#bf616a", modifiers = ["bold"] }
visualizer_background = { bg = "#050e1c" }
visualizer_stars = { fg = "#4b91cd", modifiers = ["dim"] }
visualizer_star_trail = { fg = "#14375f", modifiers = ["dim"] }
playlist = { fg = "#d8dee9" }
playlist_active = { fg = "black", bg = "#88c0d0", modifiers = ["bold"] }
playlist_index = { fg = "ansi:243" }
media_icon = { fg = "#b48ead" }
scrollbar_track = { fg = "ansi:237" }
scrollbar_thumb = { fg = "#88c0d0" }
detail_label = { fg = "#81a1c1", modifiers = ["bold"] }
detail_value = { fg = "#e5e9f0" }
status_icon = { fg = "light_green" }
status = { fg = "#d8dee9" }
key = { fg = "black", bg = "#d8dee9", modifiers = ["bold"] }
footer = { fg = "dark_gray" }
```

## Color formats

Colors are case-insensitive and can use:

- Standard names: `black`, `red`, `green`, `yellow`, `blue`, `magenta`,
  `cyan`, `gray`, `dark_gray`, and `default`.
- Bright names: `light_red`, `light_green`, `light_yellow`, `light_blue`,
  `light_magenta`, `light_cyan`, and `bright_white`.
- True color: `#ff8800` or `rgb(255, 136, 0)`.
- 256-color palette indexes: `ansi:236` or simply `236`.

Unknown colors or modifiers retain that element's built-in fallback, which keeps
the interface readable when a terminal has limited color support.

## Modifiers

Supported modifiers are `bold`, `dim`, `italic`, `underlined`, `reversed`,
`crossed_out`, `slow_blink`, `rapid_blink`, and `hidden`. Actual rendering depends
on terminal support.

## Style keys

Available keys are `app`, `border`, `logo`, `header`, `playback`, `title`,
`volume`, `tabs`, `tabs_active`, `panel_title`, `progress_track`, `progress_fill`,
`playlist`, `playlist_active`, `playlist_index`, `media_icon`, `scrollbar_track`,
`scrollbar_thumb`, `visualizer_low`, `visualizer_mid`, `visualizer_high`,
`visualizer_peak`, `visualizer_hot`, `visualizer_idle`, `details`, `detail_label`,
`detail_value`, `hint`, `status_icon`, `status`, `separator`, `key`, and `footer`.

The legacy array syntax remains valid. Its values mean foreground, background,
then modifiers:

```toml
[ui.colors]
title = ["cyan", "default", "bold"]
```

## Borders and symbols

`border_type` accepts `plain`, `rounded`, `double`, `thick`, `quadrant_inside`,
or `quadrant_outside`. Set `unicode = false` to use ASCII playback, selection,
media, scrollbar, and border-adjacent symbols where possible.

## FFT visualizer

When the terminal is at least 92 columns wide, the right pane displays a live
128-band FFT data from GStreamer's `spectrum` analyzer is wrapped around a
rotating circle. Bands rise outward from the center, rotation accelerates with
measured activity, and Unicode mode packs the radial geometry into Braille
cells. The default palette blends green, cyan, magenta, and yellow with a
bright peak glow; visualizer color keys can override the gradient. Fast attack,
smoothed release, and peak decay keep it reactive without inventing motion when
no audio is present.

Set `visualizer_sensitivity` in `[ui]` to tune response. The default is `1.25`;
typical values range from `0.7` to `2.0`. With `unicode = false`, the visualizer
uses an ASCII radial fallback. Track information is separate and appears only while
the `v` overlay is toggled on.
