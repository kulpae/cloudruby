require 'observer'
require 'logger'
begin
  require 'json/pure'
rescue LoadError
  require 'json'
end

require_relative 'soundcloud.rb'
require_relative 'mpg123player.rb'
require_relative 'gstplayer.rb'
require_relative 'ncurses_ui.rb'

class Cloudruby
  def init(q, config)
    @config = config

    # @logger = Logger.new "logfile.log"
    @logger = Logger.new STDERR
    # @logger.level = Logger::DEBUG
    @logger.level = Logger::Severity::UNKNOWN

    @cloud = SoundCloud.new "76796f79392f9398288cdac3fe3391c0", logger: @logger
    case @config[:"audio-backend"] 
    when "gstreamer"
      @player = GstPlayer.new logger: @logger, audio_params: @config[:"audio-params"]
    else
      @player = MPG123Player.new logger: @logger
    end
    @ui = NCursesUI.new self, (@config[:ncurses] || @config[:curses]), logger: @logger, audio_backend: @player

    @logger.info {"logger inited"}
    
    @player.add_observer @ui, :player_update
    @player.add_observer self
    @cloud.add_observer @ui, :cloud_update
    @logger.info {"observer assigned"}

    @cloud.load_playlist q
    @logger.info {"loaded playlist"}
    @cloud.shufflePlaylist unless @config[:"no-shuffle"]
    @logger.info {"playlist shuffled"}
    nextTrack

    trap("INT") do
      self.quit
    end
  end

  def nextTrack
    track = @cloud.nextTrack
    unless track
      puts "Nothing found"
      quit
    else
      @player.play track
    end
  end

  def prevTrack
    track = @cloud.prevTrack
    unless track
      puts "Nothing found"
      quit
    else
      @player.play track
    end
  end

  def pause
    @player.pause
  end

  def volumeUp
    @player.volume = 5
  end

  def volumeDown
    @player.volume = -5
  end
  
  def toggleMute
    @player.mute
  end

  def run
    @ui.run
  end

  # jump to next track, if current track finishes
  def update(arg)
    state = arg[:state]
    case state
    when :stop
      nextTrack
    end
  end

  def download
    @cloud.download @config[:download_dir]
  end

  # quit app and free all resources
  def quit
    @ui.close
    @player.close
    @logger.close
    exit
  end
end
