Gem::Specification.new do |s|
  s.name        = 'cloudruby'
  s.version     = '1.2.0'
  s.date        = '2015-11-04'
  s.summary     = "Ncurses player for Soundcloud tracks in Ruby"
  s.description = <<-EOF
== A soundcloud player written in Ruby 
with Ncurses for user interface and 
mpg123 for playback.

Requires the mpg123 executable to be present in the PATH.

== Usage

* lists recently uploaded tracks from soundcloud
   cloudruby

* lists all tracks that matches the keyword (here 'wearecastor')
   cloudruby wearecastor

* also works with the direct soundcloud URL
   cloudruby http://soundcloud.com/crassmix/feint-clockwork-hearts-crass
EOF
  s.author      = "Paul Koch"
  s.email       = "my.shando@gmail.com"
  s.files       = ["lib/cloudruby.rb", "lib/mpg123player.rb", "lib/ncurses_ui.rb", "lib/soundcloud.rb", "lib/gstplayer.rb"]
  s.homepage    = "https://github.com/kulpae/cloudruby"
  s.license     = 'MIT'

  s.executables << 'cloudruby'
  s.metadata    = { "issue_tracker" => "https://github.com/kulpae/cloudruby/issues" }

  s.required_ruby_version = '>= 1.9.2'
  s.add_runtime_dependency 'curses', '~> 1'
  s.add_runtime_dependency 'json_pure', '~> 1'
  s.add_runtime_dependency 'httpclient', '~> 2'
  s.requirements << 'gem gstreamer (optional)'
  s.requirements << 'mpg123 executable'
end
