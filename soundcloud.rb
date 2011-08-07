require 'cgi'
require 'open-uri'
require 'json'

class SoundCloud
  LIMIT = 100

  def initialize(client_id)
    @cid = client_id
    @random_queue = []
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
    rescue => e
      @error = {:error => e}
  end

  def nextTrack
    return @error unless @tracks
    return if @tracks.empty?
    if @tracks.is_a? Hash
      t = @tracks
    else 
      t = @tracks.sample
    end
    #puts t["user"]["username"]
    a = Track.new t, stream_url(t)
    a
  end

  private
  def stream_url(track)
    "#{track["stream_url"]}?client_id=%s" % [@cid] if track && track["stream_url"]
  end
end

class Track
  attr_accessor :url
  attr_accessor :time
  attr_accessor :title
  attr_accessor :user
  def initialize(track, stream_url)
    return unless track && track["streamable"]
    @url = stream_url
    @time = track["duration"].to_i / 1000
    @title = track["title"]
    @user = track["user"]["username"]
  end

  def to_s
    "Track \"#{title}\" by \"#{user}\" :#{url}"
  end
end
