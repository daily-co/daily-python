# Video send settings test

`send_settings_test.py` joins a Daily meeting and sends 1280x720, 30 fps VP8
video with one of the camera send settings below. While it runs, it prints the
sender's bandwidth estimate and send bitrate, so you can see how quickly each
setting ramps up.

| `-s` value            | Settings                                                                           |
|-----------------------|------------------------------------------------------------------------------------|
| `default-abr`         | Default 3 simulcast layers, adaptive bitrate (ABR) on, 1 Mbps start bitrate        |
| `abr-off`             | Default 3 layers, ABR off, 1 Mbps start bitrate                                    |
| `single-layer`        | One full-resolution layer up to 2 Mbps, ABR off, 2 Mbps start bitrate              |
| `abr-off-high-2mbps`  | Default layers with the high layer raised to 2 Mbps, ABR off, 2 Mbps start bitrate |
| `abr-on-high-2mbps`   | Same as above with ABR on                                                          |
| `abr-on-single-layer` | One full-resolution layer up to 2 Mbps, ABR on, 2 Mbps start bitrate               |

Each setting is a function in `send_settings_test.py`, so it's easy to change
one or add another.

## Usage

```
cd daily-python
# --release: keep it. A debug build is much slower and would skew the bitrate numbers.
maturin develop --release
python demos/video/send_settings_test.py -v demos/video/test_video.mp4 -m https://filipi.daily.co/bots -s single-layer
```

Then open `MEETING_URL` in a browser to watch the video. Leave the meeting with
just the two of you: with more participants, the SFU's adaptive bitrate behaves
differently.

Sending a video file needs `opencv-python`.

Options:

- `-v VIDEO`: loop a different video file.
- `-d SECONDS`: leave the meeting this many seconds after joining instead of
  waiting for Ctrl-C.

Example output:

```
Joined. Camera send settings: {...}
   2.0 s  estimate    2000 kbps  sending    1612 kbps
   4.0 s  estimate    2400 kbps  sending    1928 kbps
```

To see what the browser receives (resolution, frame rate, bitrate), open
`chrome://webrtc-internals` in Chrome while it is in the meeting.
