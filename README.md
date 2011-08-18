# CloudRuby

A soundcloud player written in Ruby with Ncurses for graphical interface and mpg123
for playback.

## Installation

Install mpg123, ruby 1.9 and ncurses with a package manager of your
destribution.

Then install the required gems.

If you are using RVM:
<pre>
  gem install ncurses
</pre>

Without RVM you need to obtain write permissions with sudo:
<pre>
  sudo gem install ncurses
</pre>

If it fails installing `ncurses`, then this step is also required:
(If using RVM, ignore `sudo`)
<pre>
  wget http://github.com/downloads/rkumar/rbcurse/ncurses-1.2.4.gem
  sudo gem install --local ncurses-1.2.4.gem
</pre>

## Usage
From the terminal start with:
<pre>
  cloudruby          # query the latest 100 tracks from soundcloud 
  cloudruby $search  # query the latest 100 tracks that match the $search keyword
  
  # play a soundcloud url directly
  cloudruby http://soundcloud.com/crassmix/feint-clockwork-hearts-crass
</pre>

Inside of Ncurses:
<table style="font-family: monospace">
<tr><th width="70px" align="left">Key</th><th>Description</th></tr>
<tr><td>q | Q        </td><td>Quit</td></tr>
<tr><td>+ | =        </td><td>Increase Volume</td></tr>
<tr><td>- | _        </td><td>Decrease Volume</td></tr>
<tr><td>n | N | Up   </td><td>Next track</td></tr>
<tr><td>p | P | Down </td><td>Previous track</td></tr>
<tr><td>m | M        </td><td>Toggle mute</td></tr>
</table>

## Screenshot

![Screenshot showing ncurses user interface](https://lh5.googleusercontent.com/-G8tsAizLZeA/TkbN9K5aAFI/AAAAAAAAAG0/EhACmmuct7s/s800/cloudruby-%2525237767fc4.png)

## Known Bugs
* Going through the playlist too fast causes silence for some time

## Author
Paul Koch [kulpae]

http://www.uraniumlane.de

## License
see LICENSE.
