# daily-python demos

Here you can find a few demos that use Daily's Python SDK:

- **audio**: Examples on how to send and receive RAW audio or WAV files.
- **deepgram**: An example showing how to use Deepgram [Text-To-Speech](https://developers.deepgram.com/docs/text-to-speech) API.
- **flask**: A demo that uses [Flask](https://flask.palletsprojects.com/) and [Celery](https://docs.celeryq.dev/) to launch multiple concurrent audio bots.
- **google**: Audio examples using Google [Speech-To-Text](https://cloud.google.com/speech-to-text) and [Text-To-Speech](https://cloud.google.com/text-to-speech) APIs.
- **gstreamer**: A media player based on [GStreamer](https://gstreamer.freedesktop.org/) that sends a video file into a meeting.
- **gtk**: A native [Gtk](https://www.gtk.org/) application that shows how to receive and render video frames for a participant.
- **openai**: A demo that takes spoken audio, converts it to text prompt, and uses [DALL-E](https://openai.com/dall-e) to generate an image.
- **pyaudio**: A demo that shows how to use [PyAudio](https://www.qt.io/qt-for-python) to record and play audio with real microphones and speakers.
- **qt**: A native [Qt](https://www.qt.io/qt-for-python) application that shows how to receive and render video frames for a participant.
- **vad**: Voice Activity Detection (VAD) examples.
- **video**: Examples on how to send and receive video or images.
- **yolo**: A demo that detects objects in a participant's video feed using [YOLOv5](https://pypi.org/project/yolov5/).

# Running

The demos have a few Python dependecies, declared in the `demos` dependency group of the `daily-python` project (see `../pyproject.toml`). They are managed with [uv](https://docs.astral.sh/uv/), which takes care of creating a virtual environment for you:

```
uv sync --group demos
```

ℹ️ `daily-python` is not included in the `demos` dependency group so you need to install it manually:

```
uv pip install daily-python
```

⚠️ It's possible that some dependencies fail to install because of missing system dependecies (e.g. `PyAudio` depends on the `portaudio` library). In those cases, it is necessary to install those dependencies manually (error messages might give hints on what system libraries are missing).

You can then run a demo with `uv run`, for example:

```
uv run python audio/wav_audio_receive.py -m YOUR_MEETING_URL -o recording.wav
```

Finally, view the demo files for more details, including how to run them.
