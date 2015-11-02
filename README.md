# CloudRuby

A soundcloud player written in Ruby with Ncurses for graphical interface and mpg123
for playback.

## Installation

Install mpg123, ruby 1.9.2, curses and json_pure with a package manager of your
destribution.

Then install the required gems.

If you are using RVM:
<pre>
  gem install curses json_pure
</pre>

Without RVM you need to obtain write permissions with sudo:
<pre>
  sudo gem install curses json_pure
</pre>

## Usage
From the terminal start with:
<pre>
  cloudruby          # query the latest 100 tracks from soundcloud (dashboard with token)
  cloudruby $search  # query the latest 100 tracks that match the $search keyword
  
  # play a soundcloud url directly
  cloudruby http://soundcloud.com/crassmix/feint-clockwork-hearts-crass
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
<tr><td>v | V        </td><td>About dialog</td></tr>
<tr><td>Spacebar     </td><td>Toggle playback</td></tr>
</table>

## User stream (also dashboard) 
By configuring an access token for your account the latest 100 tracks
from your stream are loaded, much like being logged in on soundcloud
website.

[Generate your token here](http://ninsei.thruhere.net/soundcloud.php)

## Download

With 'd' or 'D' you can download a downloadable file from soundcloud. The file
will be placed inside your download directory specified with **--download_dir** argument
or inside your **~/.cloudruby.json**. If none of these are given, the current working 
directory is used.

A track is indicated by a **[D]** in the playlist if it's downloadable.

## Screenshot

![Screenshot showing curses user interface](https://dl.dropboxusercontent.com/u/16104361/images/cloudruby-default.png)
![Screenshot showing customized curses user interface](https://dl.dropboxusercontent.com/u/16104361/images/cloudruby-custom.png)

## Config

Cloudruby can be customized through **~/.cloudruby.json** file.

### Example
```json
{
  "download_dir": "~/music",
  "token": "0-00000-00000000-000000000000000",
  "curses": {
    "colors": {
      "default": ["white", "black"],
      "playlist": ["green", "black"],
      "playlist_active": ["red", "black"],
      "progress": ["cyan", "black"],
      "progress_bar": ["blue", "white"],
      "title": ["cyan"],
      "artist": ["magenta"]
    }
  }
}
```

There are 7 different 'colors', defined with a foreground and a background color.
You can use only these colors: "black", "blue", "cyan", "green", "magenta", "red", "yellow", "white".

## Author
Paul Koch [kulpae]

http://www.uraniumlane.net/users/kulpae

Contributors [magnific0](http://www.github.com/magnific0) (user stream).

## License
see LICENSE.
