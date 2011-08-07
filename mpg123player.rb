require 'open3'

class MPG123Player
  include Observable
  def initialize()
    @pin, @pout, @perr = Open3.popen3 "mpg123 --keep-open --remote"
    @volume = 100
    @muted = false

    Thread.new do watch end
    changed
    notify_observers :state => :inited
  end

  def play(track)
    @last_track = track
    unless track.nil? || track.is_a?(Hash) && track[:error]
      @pin.puts "load #{track.url}"
      @pstate = 2
    end
    changed
    notify_observers :state => :load, :track => track
  end

  def playing?
    @pstate == 2
  end

  def pause()
    @pin.puts "pause"
  end

  def stop()
    @pin.puts "stop"
  end

  def volume= (val)
    @volume += val
    @volume = [@volume, 100].min
    @volume = [@volume, 0].max
    @pin.puts "V #{@volume}"
  end

  def volume()
    @volume
  end

  def mute
    @muted = !@muted
    @pin.puts "V 0" if @muted
    @pin.puts "V #{@volume}" unless @muted
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
  end

  private

  def watch
    while not @pout.closed?
      begin
        io = IO.select([@pout, @perr])
        #puts io.inspect
        io = io.first.first
        response = io.read_nonblock 400
        #puts ">> #{line} @@"
        lines = response.split "\n"
        lines.each do |line|
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
            notify_observers :state => :playing if @pstate == 2
            notify_observers :state => :stop if @pstate == 1
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

  def pio(io, to=STDOUT)
    begin
      to.puts io.read_nonblock 400
    rescue
    end
  end
end
