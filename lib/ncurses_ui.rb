require 'curses'

class ::Hash
  def deep_merge(second)
    merger = proc { |key, v1, v2| Hash === v1 && Hash === v2 ? v1.merge(v2, &merger) : v2 }
    self.merge(second, &merger)
  end
end

class NCursesUI
  attr_accessor :logger, :audio_backend

  def initialize cloud, options = {}, params = {}
    params.each { |key, value| send "#{key}=", value }

    if not @audio_backend
      @audio_backend = Object.new
      def @audio_backend.version
        "Audio backend: None" 
      end 
    end

    defaults = {
      :colors => {
        :default => [:cyan, :blue],
        :playlist => [:cyan, :blue],
        :playlist_active => [:white, :blue],
        :progress => [:cyan, :blue],
        :progress_bar => [:blue, :cyan],
        :title => [:cyan, :black],
        :artist => [:cyan, :black],
        :status => [:magenta, :black]
      }
    }.freeze

    @options = defaults.deep_merge(options || {})
    @cloud = cloud
    @state = :running
    @frac = 0
    @title = "None"
    @op = " "
    @time = 0
    @timeleft = 0
    @playlist = []
  end

  def run
    begin
      stdscr = Curses.init_screen
      Curses.start_color
      Colors.init @options[:colors]
      stdscr.keypad true
      Curses.nonl
      Curses.cbreak
      Curses.noecho
      Curses.curs_set 0
      Curses.timeout = 5
      @p = NProgress.new stdscr, 0, 0, :progress, :progress_bar
      @l = NPlaylist.new stdscr, 4, 0, :playlist, :playlist_active, 0, 0, @playlist
      @i = NInfobox.new self, stdscr, 4, 0, :playlist, 0, 9
      @d = NDownloadBox.new stdscr, Curses.lines-1, 0, :default, 0, 1
      @l.active = 0
      last_ch = nil
      while(@state != :close)
        ch = Curses.getch
        last_ch = ch if ch
        Curses.setpos 3, 0
        Curses.clrtoeol
        # Nutils.print stdscr, 3, 0, "Test %s" % [last_ch], :red
        case ch
        when Curses::KEY_RESIZE
          @p.resize
          @l.resize
          @i.resize
          @d.resize
          Curses.refresh
        when 110, 78, 'n', 'N', Curses::KEY_DOWN
          @cloud.nextTrack
        when 112, 80, 'p', 'P', Curses::KEY_UP
          @cloud.prevTrack
        when 113, 81, 'q', 'Q', 27, Curses::KEY_EXIT
          @cloud.quit
        when 61, 43, '=', '+'
          @cloud.volumeUp
        when 45, 95, '-', '_'
          @cloud.volumeDown
        when 109, 77, 'm', 'M'
          @cloud.toggleMute
        when 68, 100, 'd', 'D'
          @cloud.download
        when 118, 86, 'v', 'V'
          @i.visible = !@i.visible
          @l.dirty = true
        when 32, ' '
          @cloud.pause
        end

        statusLine = @status || @error

        if statusLine
          Nutils.print stdscr, 3, 0, "#{statusLine}", :status
          Curses.refresh
        end
        tr = " %s " % [Nutils.timestr(@timetotal)]
        t = " %-#{Curses.cols-tr.size-1}s%s" % [Nutils.timestr(@time), tr]
        @p.value = @frac
        @p.text = t
        @p.refresh
        Nutils.print stdscr, 1, 0, "#{@op} #{@title}", :title
        Nutils.print stdscr, 2, 0, "  by #{@username}", :artist
        @l.refresh
        @i.refresh
        @d.refresh
        stdscr.refresh
      end
    rescue => ex
    ensure
      @l.close if @l
      @p.close if @p
      stdscr.close
      Curses.echo
      Curses.nocbreak
      Curses.nl
      Curses.close_screen
      puts ex.inspect if ex
      puts ex.backtrace if ex
      # Colors.debug
    end
  end

  def cloud_update(arg)
    case arg[:state]
    when :load
      @playlist |= arg[:tracks]
      @l.list = @playlist if @l
    when :shuffle
      @playlist = arg[:tracks]
      @l.list = @playlist if @l
    when :next, :previous
      pos = arg[:position]
      @l.active = pos if @l
    when :download
      if arg[:error]
        @error = "Error: #{arg[:error]}"
      end
      if arg[:count]
        count = arg[:count]
        if(count > 0)
          @l.height = -1
          @d.visible = true
          @d.count = count
          @d.title = arg[:name] if arg[:name]
        else
          @l.height = 0
          @d.visible = false
          @d.title = ""
        end
      end
    end
  end

  def player_update(arg)

    case arg[:state]
    when :load
      track = arg[:track]
      if track.nil?
        @error = "Error: Nothing found!"
      else
        @error = nil
        @title = track["title"]
        @username = track["user"]["username"]
        @timetotal = track["duration"]
        @error = "Error: #{track[:error]}" if track[:error]
      end
    when :info
      frame = arg[:frame].to_f
      frames = frame + arg[:frameleft]
      @frac = frame/frames
      @time = arg[:time].to_i
    when :pause
      @op = "\u2161"
    when :resume, :play
      @op = "\u25B6"
    when :stop
      @op = "\u25FC"
    when :error
      @error = "Error: #{arg[:error]}"
    when :status
      if arg[:type]
        @status = "#{arg[:type]}: #{arg[:value]}"
        if @statusTimeout
          @statusTimeout.exit
        end
        @statusTimeout = Thread.new do
          sleep 5
          @status = nil
          @statusTimeout = nil
        end
      end
    end
  end

  def close
    @state = :close
  end
end

class Nutils
  def self.print(scr, row, col, text, color, width = (Curses.cols))
    width = [Curses.cols, col+width].min - col
    t = "%-#{width}s" % [scroll(text, width)]
    scr.attron(Colors.map(color)) if color
    scr.setpos row, col
    scr.addstr t
    scr.attroff(Colors.map(color)) if color
  end

  def self.scroll(text, width, offset=0)
    return unless text
    ellipsis = "*"
    t = text
    if t.size+offset > width
      t = t[offset..(width-ellipsis.size-1)] << ellipsis
    end
    t
  end

  def self.timestr(sec)
    sec = sec.to_i
    "%02d:%02d" % [sec/60, sec%60]
  end

end

class Colors
  $map = {}
  $counter = 0
  def self.init colormap = {}
    colormap.each do |key, colors|
      self.add(key, colors[0], colors[1])
    end
  end

  def self.map(key)
    $map[key] || $map[:default]
  end

  def self.add(key, fg, bg)
    Curses.init_pair $counter, ncg(fg), ncg(bg)
    $map[key] = Curses.color_pair $counter
    $counter += 1
  end

  def self.debug
    puts "colors supported: #{Curses.colors}"
    puts "map: #{$map}"
  end
  # get ncurses color constant
  def self.ncg(color)
    color = :black unless color
    Curses.const_get "COLOR_#{color.upcase}"
  end
end

class NProgress
  attr_reader :value
  attr_accessor :text
  def initialize scr, row, col, color, bar_color, width=0, value = 0, text = ""
    @width = width
    @color = color
    @bar_color = bar_color
    @row = row
    @col = col
    @winfg = Curses::Window.new 1, 1, @row, @col
    @winbg = Curses::Window.new 1, self.width, @row, @col
    @value = value
    @text = text
    refresh
  end

  def width
    [@col + @width, Curses.cols].min - @col
  end

  def value=(val)
    @value = val
    @winfg.resize(1, fgw) if fgw > 0
  end

  def refresh
    offset = fgw
    Nutils.print @winbg, 0, offset, @text[offset..-1], @color
    Nutils.print @winfg, 0, 0, @text, @bar_color if fgw > 0
    @winbg.refresh
    @winfg.refresh if fgw > 0
  end
  
  def resize
  end

  def close
    @winbg.close
    @winfg.close
  end

  private
  def fgw
    w = width() == 0 ? Curses.cols - @col : width()
    (w * @value).floor
  end
end

class NPlaylist
  attr_writer :list
  attr_accessor :dirty
  def initialize scr, row, col, color, active_color, w, h, l
    @list = l
    @row = row
    @col = col
    @width = w
    @height = h
    @color = color
    @active_color = active_color
    @apos = -1
    @win = Curses::Window.new height, width, @row, @col
    @dirty = true
    refresh
  end

  def width
    w = [@col + @width, Curses.cols].min - @col
    if w <= 0
      Curses.cols - @col + w
    else
      w
    end
  end

  def height
    h = [@row + @height, Curses.lines].min - @row
    if h <= 0
      Curses.lines - @row + h
    else
      h
    end
  end

  def width=(val)
    @width = val
    resize
    refresh
  end

  def height=(val)
    @height = val
    resize
    refresh
  end

  def active=(pos)
    @apos = pos
    @dirty = true
  end

  def resize
    @win.resize height, width
    @dirty = true
  end

  def refresh
    return unless @dirty
    if !@list.is_a?(Array) || @list.empty?
      Nutils.print @win, 1, 2, "Empty playlist", @color, width - 3
    else
      r = 1
      size = height - 2
      offset = ([[size/2.0, @apos].max, [@list.size, size].max-(size/2.0)].min - size/2.0).ceil

      @list[offset..@list.size].each do |t|
        tl = t["title"]
        if @apos == r - 1 + offset
          tl = ">#{tl}"
          color = @active_color
        else
          tl = " #{tl}"
          color = @color
        end
        tr = "[%6s]" % Nutils.timestr(t["duration"]) 
        tr = "[D]#{tr}" if t["downloadable"]
        wr = tr.size
        wl = width - 3- wr
        Nutils.print @win, r, 1, tl, color, wl+1
        Nutils.print @win, r, 2+wl, tr, color, wr
        r += 1
        if(r >= height - 1)
          # print arrow down
          break
        end
      end
    end
    @win.attron(Colors.map(@color)) if @color
    @win.box 0, 0
    @win.attroff(Colors.map(@color)) if @color
    @win.refresh
    @dirty = false
  end

  def close
    @win.close
  end
end

class NInfobox
  attr_accessor :visible
  def initialize parent, scr, row, col, color, w, h
    @parent = parent
    @scr = scr
    @row = row
    @col = col
    @color = color
    @width = w
    @height = h
    @win = Curses::Window.new height, width, @row, @col
    @visible = false
    refresh
  end

  def width
    w = [@col + @width, Curses.cols].min - @col
    if w == 0
      Curses.cols - @col
    else
      w
    end
  end

  def height
    h = [@row + @height, Curses.lines].min - @row
    if h == 0
      Curses.lines - @row
    else
      h
    end
  end

  def resize
    @win.resize height, width
  end

  def refresh
    return unless @visible
    Nutils.print @win, 1, 2, "Cloudruby v1.1", :default
    Nutils.print @win, 2, 4, "UI Toolkit: #{(Curses.const_defined?"VERSION")?Curses::VERSION : "N/A"}", :default
    Nutils.print @win, 3, 4, "#{@parent.audio_backend.version}", :default
    Nutils.print @win, 4, 4, "Ruby version: #{RUBY_VERSION}", :default
    Nutils.print @win, 5, 4, "Author: kulpae <my.shando@gmail.com>", :artist
    Nutils.print @win, 6, 4, "Website: uraniumlane.net", :title
    Nutils.print @win, 7, 4, "License: MIT", :default
    @win.attron(Colors.map(@color)) if @color
    @win.box 0, 0
    @win.attroff(Colors.map(@color)) if @color
    @win.refresh
  end

  def close
    @win.close
  end
end

class NDownloadBox
  attr_accessor :visible, :count, :title
  def initialize scr, row, col, color, w, h
    @scr = scr
    @row = row
    @col = col
    @color = color
    @width = w
    @height = h
    @win = Curses::Window.new height, width, @row, @col
    @visible = false
    @count = 0
    @title = ""
    refresh
  end

  def width
    w = [@col + @width, Curses.cols].min - @col
    if w == 0
      Curses.cols - @col
    else
      w
    end
  end

  def height
    h = [@row + @height, Curses.lines].min - @row
    if h == 0
      Curses.lines - @row
    else
      h
    end
  end

  def resize
    @win.resize height, width
  end

  def refresh
    return unless @visible
    Nutils.print @win, 0, 0, "Downloading #{@count} track#{@count > 1?"s":""} | #{title}", :default
    @win.refresh
  end

  def close
    @win.close
  end
end
