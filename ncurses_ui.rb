require 'ncurses'
require 'logger'

class NCursesUI
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
    begin
      stdscr = Ncurses.initscr
      Ncurses.start_color
      Colors.init
      Ncurses.keypad stdscr, true
      Ncurses.nonl
      Ncurses.raw
      Ncurses.cbreak
      Ncurses.noecho
      Ncurses.curs_set 0
      Ncurses::halfdelay 5
      @p = NProgress.new @stdscr, 0, 0, :cyan, :blue, Ncurses.COLS-1
      while(@state != :close)
        ch = Ncurses.getch
        #Nutils.print stdscr, 5, 0, "Test %s" % [ch], :red
        case ch
        when 110, 78
          @cloud.nextTrack
        when 112, 80
          @cloud.prevTrack
        when 113, 81
          @cloud.quit
        when 61, 43
          @cloud.volumeUp
        when 45, 95
          @cloud.volumeDown
        when 109, 77
          @cloud.toggleMute
        end

        if @error
          Nutils.print stdscr, 0, 0, "Error: #{@error}", :red
        else
          Nutils.print stdscr, 1, 0, @title, :cyan
          Nutils.print stdscr, 2, 2, "by #{@username}", :cyan
          t = " %s - %s" % [timestr(@time), timestr(@timetotal)]
          @p.value = @frac
          @p.text = t
          @p.refresh
        end
        Ncurses.refresh
      end
    rescue => ex
    ensure
      @p.close if @p
      Ncurses.echo
      Ncurses.nocbreak
      Ncurses.nl
      Ncurses.endwin
      puts ex.inspect if ex
      puts ex.backtrace if ex
    end
  end

  def cloud_update(arg)

  end

  def player_update(arg)
    case arg[:state]
    when :load
      track = arg[:track]
      if track.nil?
        @error = "Nothing found!"
      elsif track.is_a?(Hash) && track[:error]
        @error = track[:error]
      else
        @error = nil
        @title = track["title"]
        @username = track["user"]["username"]
        @timetotal = track["duration"].to_i/1000
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

class Nutils
  def self.print scr, row, col, text, fg=nil, bg=nil, width = (Ncurses.COLS-1) 
    t = "%-#{width}s" % [text]
    Ncurses.attron(Colors.map(fg, bg)) if fg
    Ncurses.mvwprintw scr, row, col, t
    Ncurses.attroff(Colors.map(fg, bg)) if fg
  end
end

class Colors
  $map = {}
  $counter = 0
  def self.init

    colors = [:black, :white, :red, :green, :yellow, :blue, :magenta, :cyan]
    self.add :white, :black
  end

  def self.map(fg, bg = nil)
    bg = :black unless bg
    pair = [fg, bg]
    unless $map[pair]
      self.add fg, bg
    end
    $map[pair]
  end

  def self.add(fg, bg)
    Ncurses.init_pair $counter, ncg(fg), ncg(bg)
    pair = [fg, bg]
    $map[pair] = Ncurses.COLOR_PAIR($counter)
    $counter += 1
  end

  # get ncurses color object
  def self.ncg(color)
    Ncurses.const_get "COLOR_#{color.upcase}"
  end
end

class NProgress
  attr_reader :value
  attr_accessor :text
  def initialize scr, row, col, fg, bg, width, value = 0, text = ""
    @width = width
    @bg = bg
    @fg = fg
    @width = Ncurses.COLS - col - 1 if @width + col > Ncurses.COLS - 1
    @winfg = Ncurses.newwin 1, 1, row, col
    @winbg = Ncurses.newwin 1, @width, row, col
    @value = value
    @text = text
    refresh
  end

  def value=(val)
    @value = val
    Ncurses.wresize @winfg, 1, fgw if fgw > 0
  end

  def refresh
    Ncurses.wbkgd @winbg, Colors.map(@fg, @bg)
    Ncurses.wbkgd @winfg, Colors.map(@bg, @fg) if fgw > 0
    Nutils.print @winbg, 0, 0, @text
    Nutils.print @winfg, 0, 0, @text if fgw > 0
    Ncurses.wrefresh @winbg
    Ncurses.wrefresh @winfg if fgw > 0
  end

  def close
    Ncurses.delwin @winbg
    Ncurses.delwin @winfg
  end

  private
  def fgw
    (@width * @value).floor
  end
end
