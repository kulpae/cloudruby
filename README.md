# CloudRuby

A soundcloud player written in Ruby with Ncurses for graphical interface and mpg123
for playback.


## Usage
From the terminal start with:
<pre>
  cloudruby          # query the latest 500 tracks from soundcloud 
  cloudruby $search  # query the latest 500 tracks that match the $search keyword
</pre>

Inside of NCurses:
<table style="font-family: monospace">
<tr><th width="50px" align="left">Key</th><th>Description</th></tr>
<tr><td>q | Q</td><td>Quit</td></tr>
<tr><td>+ | =</td><td>Increase Volume</td></tr>
<tr><td>- | \_</td><td>Decrease Volume</td></tr>
<tr><td>n | N</td><td>Next track</td></tr>
<tr><td>m | M</td><td>Toggle mute</td></tr>
</table>
