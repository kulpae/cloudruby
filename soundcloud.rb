require 'cgi'
require 'open-uri'
require 'json'

class SoundCloud
  include Observable
  LIMIT = 100

  def initialize(client_id)
    @cid = client_id
    @playlist_pos = -1
  end

  def load_playlist(search = nil, offset = 0)
    search = "" unless search && !search.empty?
    if search =~ /\s*http(s)?:\/\/(www.)?soundcloud.com.*/
      url = "http://api.soundcloud.com/resolve.json?url=%s&client_id=%s" % [CGI.escape(search), @cid]
    else
      url = "http://api.soundcloud.com/tracks.json?client_id=%s&filter=streamable&limit=%d&offset=%d&q=%s" \
        % [@cid, LIMIT, offset, CGI.escape(search)]
    end
    c = open(url) do |io|
      io.readlines
    end.join
    @tracks = JSON.parse c
    @tracks = [@tracks] if @tracks.is_a? Hash
    @tracks.map! do |t|
      t["mpg123url"] = stream_url t
      t
    end
    changed
    notify_observers :state =>:load, :tracks => @tracks
    rescue => e
      @error = {:error => e}
  end

  def shufflePlaylist
    @tracks.shuffle!
    changed
    notify_observers :state => :shuffle
  end

  def nextTrack
    return @error unless @tracks
    return if @tracks.empty? || @tracks.nil?
    if @tracks.is_a? Hash
      t = @tracks
    else 
      @playlist_pos += 1
      @playlist_pos -= @tracks.size if @playlist_pos >= @tracks.size
      t = @tracks[@playlist_pos]
      changed
      notify_observers :state => :next, :position => @playlist_pos
    end
    t
  end

  def prevTrack
    return @error unless @tracks
    return if @tracks.empty?
    if @tracks.is_a? Hash
      t = @tracks
    else 
      @playlist_pos -= 1
      @playlist_pos += @tracks.size if @playlist_pos < 0
      t = @tracks[@playlist_pos]
      changed
      notify_observers :state => :previous, :position => @playlist_pos
    end
    t
  end

  private

  def stream_url(track)
    "#{track["stream_url"]}?client_id=%s" % [@cid] if track && track["stream_url"]
  end
end

