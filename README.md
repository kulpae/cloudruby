# CloudRuby

A soundcloud player written in Ruby with Ncurses for graphical interface and mpg123
for playback.

## Installation

Install mpg123, ruby 1.9 and ncurses with a package manager of your
destribution.

Then install the required gems.
If you are using rvm:
``` gem install rbcurse --pre
    gem install ncurses```

otherwise:
``` sudo gem install rbcurse --pre
    sudo gem install ncurses```

If it fails installing `ncurses`, then this step is also required:
``` wget http://github.com/downloads/rkumar/rbcurse/ncurses-1.2.4.gem
    sudo gem install --local ncurses-1.2.4.gem ```

## Usage
From the terminal start with:
<pre>
  cloudruby          # query the latest 100 tracks from soundcloud 
  cloudruby $search  # query the latest 100 tracks that match the $search keyword
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
