require 'cgi'
require 'open-uri'
require 'httpclient'

require 'json'

class SoundCloud
  include Observable
  LIMIT = 100
  attr_accessor :logger

  def initialize client_id, params = {}
    params.each { |key, value| send "#{key}=", value }
    @cid = client_id
    @playlist_pos = -1
    @download_queue = []
    @dthread = Thread.new do downloader end
    @dthread.run
  end

  def load_playlist(args = nil, offset = 0)
    args = [] unless args && !args.empty?
    @tracks = []
    args.each do |search|
        if search =~ /\s*http(s)?:\/\/(www.)?soundcloud.com.*/
            url = "https://api.soundcloud.com/resolve.json?url=%s&client_id=%s" % [CGI.escape(search), @cid]
        else
            url = "https://api.soundcloud.com/tracks.json?client_id=%s&filter=streamable&limit=%d&offset=%d&q=%s" \
                % [@cid, LIMIT, offset, CGI.escape(search)]
        end
        c = open(url) do |io|
            io.readlines
        end.join
        trx = JSON.parse c
        trx = [trx] if trx.is_a? Hash
        trx.map! do |t|
            t["mpg123url"] = client_url t['stream_url']
            t["download"] = client_url t['download_url']
            t["duration"] = t["duration"].nil? ? 0 : t["duration"].to_i/1000
            t["bpm"] = t["bpm"].nil? ? 0 : t["bpm"].to_i
            t[:error] = "Not streamable" if t["stream_url"].nil?
            t["searchterm"] = search
            @tracks << t
        end
    end
    changed
    notify_observers :state =>:load, :tracks => @tracks
    rescue => e
      @error = {:error => e}
  end

  def shufflePlaylist
    return unless @tracks.respond_to? "shuffle!"
    @tracks.shuffle!
    changed
    notify_observers :state => :shuffle, :tracks => @tracks
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

  def download(target_dir)
    return @error unless @tracks
    return if @tracks.empty?
    if @tracks.is_a? Hash
      t = @tracks
    else 
      t = @tracks[@playlist_pos]
    end
    unless t["download"].nil? || t["download"].size == 0
      filename = "#{t["permalink"]}.#{t["original_format"]}"
      path = File.join(target_dir, filename)
      pair = [path, t['download']]
      @logger.debug {"download queued: #{pair}"}
      @download_queue << pair unless @download_queue.include? pair

      changed
      notify_observers :state => :download, :count => @download_queue.size
    else
      changed
      notify_observers :state => :status, :type => "Download", :value => "Not granted by content owner"
    end
  end

  # download thread
  def downloader
    loop do
      while d = @download_queue.shift
        path = d[0]
        uri = d[1]
        size = @download_queue.size
        changed
        notify_observers :state => :download, :name => path, :count => size
        begin
          path = File.expand_path path
          File.open(path, "wb") do |file|
            cl = HTTPClient.new
            cl.redirect_uri_callback = ->(uri, res) {
              res.header['location'][0]
            }
            cl.get_content(uri) do |chunk|
              file.write(chunk)
            end
          end
        rescue Exception => e
          @logger.error {"Download error: #{e}"}
          changed
          notify_observers :state => :error, :error => e
        end
      end
      sleep 5
    end
  rescue => e
    changed
    @logger.error {"Downloader thread: #{e}"}
    notify_observers :state => "error", :error => e
  end

  private

  def client_url(url)
    "#{url}?client_id=%s" % [@cid] if url
  end
end

