# Colors

CloudRuby's Ratatui interface reads foreground and background colors from the
TOML configuration file:

```toml
[colors]
foreground = "white"
background = "black"
```

Supported values are `default`, `black`, `red`, `green`, `yellow`, `blue`,
`magenta`, `cyan`, and `white`. Values are case-insensitive. An unknown value
falls back to the terminal default, so the application remains usable with an
older or hand-edited configuration.

The selected row uses the terminal's reversed style. Status and progress
indicators use semantic colors chosen by the application.
