# CloudRuby

A soundcloud player written in Ruby with Ncurses for graphical interface and mpg123
for playback.

## Installation

Install mpg123, ruby 1.9.2+, curses and json_pure with a package manager of your
distribution.

Then install the required gems.

If you are using RVM:
<pre>
  gem install curses json_pure httpclient
</pre>

Without RVM you need to obtain write permissions with sudo:
<pre>
  sudo gem install curses json_pure httpclient open-uri
</pre>

## Usage
From the terminal start with:
<pre>
  cloudruby [--config-args] [(search terms|track urls)]

  # Examples

  cloudruby          # query the latest 100 tracks from soundcloud
  cloudruby $search  # query the latest 100 tracks that match the $search keywords

  ## play a soundcloud url directly
  cloudruby http://soundcloud.com/crassmix/feint-clockwork-hearts-crass

  ## create a playlist from arguments (urls and/or grouped search keywords)
  cloudruby --no-shuffle=true https://soundcloud.com/adapt77/auto-ok-ecophal manicanparty heart "sellorekt thunderbolt" https://soundcloud.com/wearesoundspace/premiere-malbetrieb-ghetto "elektroschneider lenny"
  > search terms are grouped when quoted or adjacent to urls or quoted strings

  ## create a playlist from a pipe
  > stdin is also parsed as arguments if piped into the process
  cloudruby --no-shuffle=true < local-playlist-filename
  printf "sellorekt iris\nmanicanparty eyes" | cloudruby
</pre>

Shortcuts:
<table style="font-family: monospace">
<tr><th width="160px" align="left">Key</th><th>Description</th></tr>
<tr><td>ESC | q | Q        </td><td>Quit</td></tr>
<tr><td>+ | =        </td><td>Increase volume</td></tr>
<tr><td>- | _        </td><td>Decrease volume</td></tr>
<tr><td>n | N | Up   </td><td>Next track</td></tr>
<tr><td>p | P | Down </td><td>Previous track</td></tr>
<tr><td>m | M        </td><td>Toggle mute</td></tr>
<tr><td>d | D        </td><td>Download file</td></tr>
<tr><td>v | V        </td><td>Info dialog</td></tr>
<tr><td>Spacebar     </td><td>Toggle playback</td></tr>
</table>

More detailed information can be found in the `doc` folder.

## Download

With 'd' or 'D' you can download a downloadable file from soundcloud. The file
will be placed inside your download directory specified with `--download_dir` argument
or inside your `~/.cloudruby.json`. If none of these are given, the current working
directory is used.

A track is indicated by a **[D]** in the playlist if it's downloadable.

## Screenshots

![Screenshot showing curses user interface](https://www.dropbox.com/s/j6uuqf56sgb53tw/cloudruby-default.png?raw=1)
![Screenshot showing customized curses user interface](https://www.dropbox.com/s/3re0xiqkd2403to/cloudruby-custom.png?raw=1)
![Screenshot showing customized curses user interface](https://www.dropbox.com/s/kfiu4ve85jsxh04/cloudruby-styling.png?raw=1)

## Config

Cloudruby can be customized through `~/.cloudruby.json` file.

### Example
```json
{
  "download_dir": "~/music",
  "audio-backend": "mpg123",
  "curses": {
    "colors": {
      "default": ["white", "black"],
      "playlist": ["green", "black"],
      "playlist_active": ["red", "black"],
      "progress": ["cyan", "black"],
      "progress_bar": ["blue", "white"],
      "title": ["cyan"],
      "artist": ["magenta"],
      "status": ["red"]
    }
  }
}
```

Read more about styling [here](doc/colors.md)

## Maintainer
* [Paul Koch [kulpae]](http://www.uraniumlane.net/users/kulpae)

## Contributors
* [magnific0](http://www.github.com/magnific0)


## License
see LICENSE.
