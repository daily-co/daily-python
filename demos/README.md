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

The demos have a few Python dependecies. To keep things clean, it's always a
good idea to use a virtual environment:

```
python3 -m venv .venv
source .venv/bin/activate
```

Once the virtual environment is activated you can install the dependencies via
`pip`:

```
pip3 install -r requirements.txt
```

ℹ️ `daily-python` is not included in the `requirements.txt` file so you need to
install it manually:

```
pip3 install daily-python
```

⚠️ It's possible that some requirements fail to install because of missing system
dependecies (e.g. `PyAudio` depends on the `portaudio` library). In those cases,
it is necessary to install those dependencies manually (error messages might
give hints on what system libraries are missing). Another alternative is to
remove the conflicting dependecies from `requirements.txt`.

Finally, view the demo files for more details, including how to run them.
