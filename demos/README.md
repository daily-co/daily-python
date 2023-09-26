# daily-python demos

Here you can find a few demos that use Daily's Python SDK:

- audio: Two demos showing how to work with audio and daily-python. One sends audio into a meeting from a file; the other receives audio from a meeting and records it.
- google: Two more audio examples: one that receives audio from a meeting and generates text, another that sends audio to a meeting from a text file.
- gtk: Shows how to receive and render video frames for a participant.
- openai: Shows how to send images to a meeting from a virtual camera device. This example takes spoken audio, converts it to text, and uses DALL-E to generate an image.

# Running

It is assumed you have `daily-python` installed.

Some of the demos need additional Python packages you can install via `pip`:

```
pip3 install -r requirements.txt
```

ℹ️ It's a good idea to install these dependencies in a virtual environment.
