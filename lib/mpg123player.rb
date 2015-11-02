require 'open3'
begin
  require 'httpclient'
rescue LoadError
end

class MPG123Player
  include Observable
  attr_reader :error, :paused
  attr_accessor :logger

  def version
    return "Audio backend: #{`mpg123 --version`.strip}"
  end

  def initialize params = {}
    params.each { |key, value| send "#{key}=", value }
    unless defined? HTTPClient
      puts "Mpg123 requires HTTPClient to work"
      exit false
    end

    @volume = 100
    @muted = false
    @logger.info version
    @inqueue = []
    begin
      @pin, @pout, @perr = Open3.popen3 "mpg123 --keep-open --remote"
      Thread.new do mpg123read end
      Thread.new do mpg123send end
      changed
      notify_observers :state => :inited
    rescue => err
      @error = err
      changed
      notify_observers :state => :error, :error => err
    end
  end

  def play(track)
    @last_track = track
    unless track.nil? || track.is_a?(Hash) && track[:error]
      # mpg123puts "load #{track["mpg123url"]}"
      strhead   = HTTPClient.new.head(track['mpg123url'])
      streamuri = strhead.header['Location'][0]
      streamuri.sub! 'https', 'http'
      mpg123puts "load #{streamuri}"
      @pstate = 2
    end
    changed
    notify_observers :state => :load, :track => track
  rescue => err
    @error = err
    changed
    notify_observers :state => :error, :error => err
  end

  def playing?
    @pstate == 2 && !@paused
  end

  def pause()
    @paused = !@paused
    mpg123puts "pause"
  rescue => err
    @error = err
    changed
    notify_observers :state => :error, :error => err
  end

  def stop()
    mpg123puts "stop"
  rescue => err
    @error = err
    changed
    notify_observers :state => :error, :error => err
  end

  def volume= (val)
    @volume += val
    @volume = [@volume, 100].min
    @volume = [@volume, 0].max
    mpg123puts "V #{@volume}"
  rescue => err
    @error = err
    changed
    notify_observers :state => :error, :error => err
  end

  def volume()
    @volume
  end

  def mute
    @muted = !@muted
    mpg123puts "V 0" if @muted
    mpg123puts "V #{@volume}" unless @muted
  rescue => err
    @error = err
    changed
    notify_observers :state => :error, :error => err
  end

  def muted?
    @muted
  end

  def close()
    @perr.close
    @pout.close
    @pin.close
    changed
    notify_observers :state => :closed
  rescue => err
    @error = err
    changed
    notify_observers :state => :error, :error => err
  end

  def mpg123puts(out)
    @inqueue << out
    @logger.debug {">> #{out}"} #
  end

  private

  def mpg123send
    while not @pin.closed?
      begin
        last_cmd = nil
        @logger.debug "queue size #{@inqueue.size}"
        @inqueue.select! do |cmd|
          if cmd[0,4] == "load"
            last_cmd = cmd
            false
          else
            true
          end
        end
        @inqueue << last_cmd unless last_cmd.nil?
        while out = @inqueue.shift
          @pin.puts out
        end
        select(nil, nil, nil, 0.4)
      rescue 
      end
    end
  end

  def mpg123read
    while not @pout.closed?
      begin
        io = IO.select([@pout, @perr])
        io = io.first.first
        response = io.read_nonblock 1024
        lines = response.split "\n"
        lines.each do |line|
          @logger.debug {"<< #{line}"} #
          if line =~ /@F\s(\S*)\s(\S*)\s(\S*)\s(\S*)\s*/
            changed
            notify_observers :state => :info, 
              :frame => $1.to_i, 
              :frameleft => $2.to_i, 
              :time => $3,
              :timeleft => $4
          elsif line =~ /@P\s(\S*)\s*/
            @pstate = $1.to_i
            changed
            if @paused 
              if @pstate == 2
                notify_observers :state => :resume
                @paused = false
              elsif @pstate == 1
                notify_observers :state => :pause 
              end
            else
              notify_observers :state => :play if @pstate == 2
              notify_observers :state => :stop if @pstate == 1
            end
          elsif line =~ /@E.*Unfinished command:.*/
            play @last_track
          else
            #puts "don't know #{line}"
          end
        end
      rescue
      end
    end
  end
end
