# daily-python demos

Here you can find a few demos that use Daily's Python SDK:

-   **audio**: Examples on how to send and receive RAW audio or WAV files.
-   **flask**: A demo that uses [Flask](https://flask.palletsprojects.com/) and [Celery](https://docs.celeryq.dev/) to launch multiple concurrent audio bots.
-   **google**: Audio examples using Google [Speech-To-Text](https://cloud.google.com/speech-to-text) and [Text-To-Speech](https://cloud.google.com/text-to-speech) APIs.
-   **gtk**: A native [Gtk](https://www.gtk.org/) application that shows how to receive and render video frames for a participant.
-   **openai**: A demo that takes spoken audio, converts it to text prompt, and uses [DALL-E](https://openai.com/dall-e) to generate an image.
-   **yolo**: A demo that detects objects in a participant's video feed using [YOLOv5](https://pypi.org/project/yolov5/).

# Running

It is assumed you have `daily-python` installed.

Some of the demos need additional Python packages you can install via `pip`:

```
pip3 install -r requirements.txt
```

ℹ️ It's a good idea to install these dependencies in a virtual environment.

View the demo files for more details, including how to run them.
