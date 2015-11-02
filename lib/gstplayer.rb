require 'gst'

class GstPlayer
  include Observable
  attr_reader :error, :paused
  attr_accessor :logger, :audio_params

  def version
    as = nil
    unless @pipeline.nil?
      as = @pipeline.get_property "audio-sink"
      as = ", Sink: #{as.name}"
    end
    return "Audio backend: Gstreamer #{Gst.version.join('.')}#{as}"
  end

  def initialize params = {}
    params.each { |key, value| send "#{key}=", value }

    @logger.info version
    @inqueue = []
    begin
      @pipeline = Gst::ElementFactory.make("playbin", "cloudruby_pipeline")
      if @pipeline.nil?
        puts "Cannot initialize playing pipeline. Is gstreamer installed?"
        exit false
      end
      @pipeline.set_property "buffer-size", 16 * 1024 * 1024
      if @audio_params
        @audio_params.each do |key, value|
          case key
          when :"audio-sink"
            sink = Gst::ElementFactory.make(value, "#{value}")
            @pipeline.set_property "audio-sink", sink unless sink.nil?
          when :"buffer-duration", :"buffer-size", :"mute", :"volume"
            @pipeline.set_property key, value
          end
        end
      end

      changed
      notify_observers :state => :inited
      runMainLoop
    rescue => err
      @error = err
      changed
      notify_observers :state => :error, :error => err
    end
  end

  def play(track)
    @last_track = track
    unless track.nil? || track.is_a?(Hash) && track[:error] && !Gst.valid_uri?(track["mpg123url"])
      gstPlayUrl track["mpg123url"]
    end
    changed
    notify_observers :state => :load, :track => track
  rescue => err
    @error = err
    changed
    notify_observers :state => :error, :error => err
  end

  def playing?
    @pipeline.playing?
  end

  def pause()
    @paused = !@paused
    if @paused
      @pipeline.pause
    else
      @pipeline.play
    end
  rescue => err
    @error = err
    changed
    notify_observers :state => :error, :error => err
  end

  def stop()
    @playbin.stop
  rescue => err
    @error = err
    changed
    notify_observers :state => :error, :error => err
  end

  def volume= (val)
    @volume = volume
    @volume += val
    @volume = [@volume, 100].min
    @volume = [@volume, 0].max
    @pipeline.volume = @volume/100.0
    changed
    notify_observers :state => :status, :type => "Volume", :value => "#{@volume}%"
  rescue => err
    @error = err
    changed
    notify_observers :state => :error, :error => err
  end

  def volume
    (@pipeline.get_property("volume") * 100).to_int
  end

  def mute
    @pipeline.set_property "mute", !@pipeline.get_property("mute")
    changed
    notify_observers :state => :status, :type => "Volume", :value => "#{muted? ? 0 : @volume}%"
  rescue => err
    @error = err
    changed
    notify_observers :state => :error, :error => err
  end

  def muted?
    @pipeline.get_property "mute"
  end

  def close()
    @pipeline.stop
    @mainloop.quit
    changed
    notify_observers :state => :closed
  rescue => err
    @error = err
    changed
    notify_observers :state => :error, :error => err
  end

  def gstPlayUrl(url)
    @pipeline.stop
    @pipeline.uri = url
    @pipeline.play
  end

  private

  def runMainLoop
    @logger.debug {"listen to gst pipeline"}
    @mainloop = GLib::MainLoop.new nil, true

    # Add custom tasks
    GLib::Timeout.add_seconds(1) do
      pos_res, position = @pipeline.query_position(Gst::Format::TIME)
      dur_res, duration = @pipeline.query_duration(Gst::Format::TIME)
      # @pipeline.query
      duration = (duration / 1000000.0 / 1000).to_int
      position = (position / 1000000.0 / 1000).to_int
      if position > 0 && duration > 0 && duration > position
        changed
        notify_observers :state => :info, 
          :frame => position, 
          :frameleft => duration-position, 
          :time => position,
          :timeleft => position-duration
      end
      true
    end

    bus = @pipeline.bus
    bus.add_watch do |bus, message|
      raise "message nil" if message.nil?

      case message.type
      when Gst::MessageType::EOS
        changed
        notify_observers :state => :stop
      when Gst::MessageType::WARNING
        warning, debug = message.parse_warning
        changed
        notify_observers :state => :error, :error => warning.message
      when Gst::MessageType::ERROR
        error, debug = message.parse_error
        changed
        notify_observers :state => :error, :error => error.message
      when Gst::MessageType::BUFFERING
        percent = message.parse_buffering
        changed
        notify_observers :state => :status, :type => "Buffering", :value => "#{percent}%"
      when Gst::MessageType::STATE_CHANGED
        error, state = message.parse_state_changed
        case state
        when Gst::State::PAUSED
          changed
          notify_observers :state => :pause
        when Gst::State::PLAYING
          changed
          notify_observers :state => :play
        end
      else
        @logger.info {"bus event: #{message.type.name} #{message.type} #{message.structure}"}
      end
      true
    end

    # @pipeline.play
    context = @mainloop.context
    thread = Thread.new do
      while @mainloop.running? do
        context.iteration false
        sleep 0.05
      end
    end

  end

end
