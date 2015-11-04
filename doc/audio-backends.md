
#Audio Backends

Cloudruby is able to work with different audio backends.
At the moment only mpg123 and gstreamer are supported.

In order to set the desired audio backend, following lines have to be appended to the config file:

_~/.cloudruby.json_
```json
{
  "audio-backend": "<audio-backend>",
}
```

where `<audio-backend>` is:
 * mpg123
 * gstreamer

### Volume scale

As every sound adapter can have a different volume curves (e.g. linear, logarithmic) cloudruby accepts a curvature paramter for the volume curve.

```json
{
	"audio-backend": "gstreamer",
	"audio-params": {
		"audio-sink": "pulsesink",
		"volume-curvature": 3
	}
}
```

The `curvature` parameter is simply an exponent of the volume. So having the `curvature` = 1 makes a linear volume curve. A `curvature` = 3 makes the same approximation to a logarithmic curve, as pulseaudio is using (7.0), so that the volume percentage of cloudruby equals the volume in the pulseaudio mixer.

###GStreamer requirements

GStreamer requires following (or similar) packages installed:

* gstreamer (gem)
* gstreamer (os)
* gst-plugins-base (os, for alsa)
* gst-plugins-good (os, for pulseaudio, oss, souphttpsrc (!))
* gst-plugins-ugly (os, for mp3  decoding)

The OS packages may be named differently on your operation system and/or distribution. These are for Arch Linux. You have to find the required packages by youself otherwise. The main point is, that these packages provide following gstreamer plugins:

* playbin
* \*sink (e.g. pulsesink, alsasink, oss4sink, jacksink, openalsink, ...)
* mad (mp3 decode)
* souphttpsrc or neonhttpsrc (http streaming)

###GStreamer options

__GStreamer__ accepts parameters in the following format, which are optional:

```json
{
  "audio-backend": "gstreamer",
  "audio-params": {
    "audio-sink": "autosink",
    "volume": 1.0,
    "mute": false,
    "buffer-size": 16777216,
    "buffer-duration": -1
  }
}
```


By default gstreamer takes the best audio sink available and works with default parameters without configuration.

When __v__ is pressed, the currently utilized audio backend and the sink (in case of gstreamer) are displayed.
