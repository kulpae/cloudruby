
#Styling cloudruby

If your terminal supports more than 8 colors and your ncurses version supports 256 colors (since 6.0+), you can change the colors of the player.

Also cloudruby uses the same font as your terminal emulator. With some changes it can be made looking very slick ;)

### Styling of the UI components
In your `~/.cloudruby.json` you can change the colors of the UI components.
Every component uses one ore more style definitions.
Every style definition has a foreground and a background color, as well as any amount of font attributes.

```lisp
  "playlist_active": [<fg-color>, <bg-color>, <attr-1>, <attr-2>, ..., <attr-n>]
```

e.g.:

```json
  "playlist_active": ["red", "black", "bold", "underline"]
```
### Style definitions
This is a list of the currently available color-pairs:

* default
* playlist
* playlist_active
* progress
* progress_bar
* buffer_bar
* title
* artist
* status

These definitions reside inside of a `curses.colors` array, e.g.:

```json
{
	"curses": {
		"colors": {
			"default": [-1, -1],
			"artist": ["red", "black", "bold"],
		}
	}
}
```

### Color definitions

There are 9 default colors:

* black
* white
* blue
* red
* green
* yellow
* cyan
* magenta
* -1

The special color `-1` represents the default terminal color. On some terminals it can be used to produce a transparent effect.

Additionally a palette of color aliases or new color definitions can be provided to enhance the color diversity.

The color palette accepts color values of the terminal (same as used for `tput setaf`, e.g. 33) or rgb triples. The later redefines the terminal colors even after leaving cloudruby, though. So be cautious. In that case you can type `reset` to restore the color definitions. The color triple contains values for red, green, blue parts of a color with every part ranging from 0 to 1000.

Example palette:

```json
{
	"curses": {
		"palette": {
			"orange": [984, 545, 0],
			"yellow": 220,
			"flame": 160,
		}
 	}
}
```

It is advised not to use color triples, but instead to find a different similar terminal color.

### Attributes
This is a list of the currently available color attributes:

* bold
* underline
* blink
* dim
* normal
* standout
* reverse

Every terminal emulator interprets these attributes in a different way or not at all, though.


### Example definition

This definitions produces the result shown in the screenshot below: 

```json
{
   "curses": {
    "colors": {
      "default": ["white", "panel-black"],
      "playlist": ["peanut", -1],
      "playlist_active": ["orange", "black", "bold"],
      "progress": ["orange", "darkgray"],
      "progress_bar": ["lust", "black", "underline", "bold", "reverse"],
      "buffer_bar": ["orange", "panel"],
      "title": ["flame", "panel-black", "bold"],
      "artist": ["crimson", "panel-black"],
      "status": ["pale", "panel-black"]
    },
    "palette": {
      "orange": 202,
      "flame": 160,
      "panel-black": 233,
      "panel": 236,
      "darkgray": 232,
      "lust": 166,
      "crimson": 161,
      "pale": 59,
      "peanut": 1
    }
  }
}
```

![Styled cloudruby](https://www.dropbox.com/s/kfiu4ve85jsxh04/cloudruby-styling.png?raw=1)