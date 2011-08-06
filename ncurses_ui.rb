require 'ncurses'
require 'rbcurse'
require 'rbcurse/rprogress'
require 'logger'

class NCursesUI
  include RubyCurses
  include RubyCurses::Utils
  def initialize cloud
    @cloud = cloud
    $log = Logger.new STDERR
    $log.level = Logger::WARN
    @state = :running
    @frac = 0
    @title = "None"
    @time = 0
    @timeleft = 0
  end

  def run
    #begin end while(@state != :close)
    #return
    begin
      VER::start_ncurses
      @win = VER::Window.root_window#new :left => 0, :top => 0, :width => 0, :height => 0
      @form = Form.new @win
      @progress = Progress.new @form, :fraction => 0.3, :text => "30%", :bgcolor => :white, :color => :red, :width => Ncurses.COLS - 1
      @progress.move 3,0
      while(@state != :close)
        ch = keycode_tos @win.getch
        #@win.printstring 4,0, "%-#{Ncurses.COLS-1}s" % ["Pressed #{ch}"], ColorMap.get_color(:cyan)
        case ch
        when "-1"
        when /n|N/
          @cloud.nextTrack
        when /Q|q/
          @cloud.quit
        when /\+|=/
          @cloud.volumeUp
        when /-|_/
          @cloud.volumeDown
        when /m|M/
          @cloud.toggleMute
        end
        if @error
          @win.printstring  2, 0, "%-#{Ncurses.COLS-1}s" % ["Error: #{@error}"], ColorMap.get_color(:red)
        else
          @win.printstring  1, 0, "%-#{Ncurses.COLS-1}s" % [@username], ColorMap.get_color(:yellow)
          t = "%s -  %02d%% - %s" % [timestr(@time), @frac*100, timestr(@timetotal)]
          @win.printstring  2, 2, "%-#{Ncurses.COLS-1}s" % [@title], ColorMap.get_color(:yellow)
        end
        @progress.fraction @frac
        @progress.text t
        @form.repaint
        @win.wmove 0,0
        @win.wrefresh
        sleep 0.05
      end
    rescue => ex
    ensure
      @win.destroy
      VER::stop_ncurses
      puts ex.inspect if ex
      puts ex.backtrace if ex
    end
  end

  def update(arg)
    #puts arg
    case arg[:state]
    when :load
      track = arg[:track]
      if track.is_a?(Hash) && track[:error]
        @error = track[:error]
      elsif track.nil?
        @error = "Nothing found!"
      else
        @error = nil
        @title = track.title
        @username = track.user
        @timetotal = track.time
      end
    when :info
      frame = arg[:frame].to_f
      frames = frame + arg[:frameleft]
      @frac = frame/frames
      @time = arg[:time].to_i
    end
  end

  def timestr(sec)
    sec = sec.to_i
    "%02d:%02d" % [sec/60, sec%60]
  end

  def close
    @state = :close
  end
end
