#
# This demo will join a Daily meeting and record the meeting audio into standard
# output. The recorded audio will have a sample rate of 16000, 16-bit per sample
# and mono audio channel.
#
# Usage: python3 raw_audio_receive.py -m MEETING_URL > FILE.raw
#
# The following example shows how to send back the recorded audio using a
# GStreamer pipeline and raw_audio_send.py:
#
# gst-launch-1.0 -q filesrc location=FILE.raw ! \
#    rawaudioparse num-channels=1 pcm-format=s16le sample-rate=16000 ! \
#    fdsink fd=1 sync=true | python3 raw_audio_send.py -m MEETING_URL
#

import argparse
import sys
import threading

from daily import *


class ReceiveAudioApp:
    def __init__(self):
        self.__speaker_device = Daily.create_speaker_device(
            "my-speaker",
            sample_rate=16000,
            channels=1
        )
        Daily.select_speaker_device("my-speaker")

        self.__client = CallClient()
        self.__client.update_subscription_profiles({
            "base": {
                "camera": "unsubscribed",
                "microphone": "subscribed"
            }
        })

        self.__app_quit = False
        self.__app_error = None

        self.__start_event = threading.Event()
        self.__thread = threading.Thread(target=self.receive_audio)
        self.__thread.start()

    def on_joined(self, data, error):
        if error:
            print(f"Unable to join meeting: {error}")
            self.__app_error = error
        self.__start_event.set()

    def run(self, meeting_url):
        self.__client.join(meeting_url, completion=self.on_joined)
        self.__thread.join()

    def leave(self):
        self.__app_quit = True
        self.__thread.join()
        self.__client.leave()
        self.__client.release()

    def receive_audio(self):
        self.__start_event.wait()

        if self.__app_error:
            print(f"Unable to receive audio!")
            return

        while not self.__app_quit:
            # Read 100ms worth of audio frames.
            buffer = self.__speaker_device.read_frames(1600)
            if len(buffer) > 0:
                sys.stdout.buffer.write(buffer)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required=True, help="Meeting URL")
    args = parser.parse_args()

    Daily.init()

    app = ReceiveAudioApp()

    try:
        app.run(args.meeting)
    except KeyboardInterrupt:
        print("Ctrl-C detected. Exiting!", file=sys.stderr)
    finally:
        app.leave()


if __name__ == '__main__':
    main()
