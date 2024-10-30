#
# This demo will join a Daily meeting and send raw audio received through the
# standard input. The audio format is required to have 16-bit per sample.
#
# Usage: python3 raw_audio_send.py -m MEETING_URL
#
# The following example sends audio from a GStreamer pipeline:
#
# gst-launch-1.0 -q audiotestsrc is-live=true samplesperbuffer=160 ! \
#    audio/x-raw,rate=16000,channels=1,format=S16LE ! \
#    fdsink fd=1 sync=true | python3 raw_audio_send.py -m MEETING_URL
#

import argparse
import sys
import threading

from daily import *


SAMPLE_RATE = 16000
NUM_CHANNELS = 1
BYTES_PER_SAMPLE = 2


class SendAudioApp:
    def __init__(self, sample_rate, num_channels):
        self.__sample_rate = sample_rate
        self.__num_channels = num_channels

        self.__mic_device = Daily.create_microphone_device(
            "my-mic", sample_rate=sample_rate, channels=num_channels
        )

        self.__client = CallClient()

        self.__client.update_subscription_profiles(
            {"base": {"camera": "unsubscribed", "microphone": "unsubscribed"}}
        )

        self.__app_quit = False
        self.__app_error = None

        self.__start_event = threading.Event()
        self.__thread = threading.Thread(target=self.send_raw_audio)
        self.__thread.start()

    def on_joined(self, data, error):
        if error:
            print(f"Unable to join meeting: {error}")
            self.__app_error = error
        self.__start_event.set()

    def run(self, meeting_url):
        self.__client.join(
            meeting_url,
            client_settings={
                "inputs": {
                    "camera": False,
                    "microphone": {"isEnabled": True, "settings": {"deviceId": "my-mic"}},
                }
            },
            completion=self.on_joined,
        )
        self.__thread.join()

    def leave(self):
        self.__app_quit = True
        self.__thread.join()
        self.__client.leave()
        self.__client.release()

    def send_raw_audio(self):
        self.__start_event.wait()

        if self.__app_error:
            print(f"Unable to send audio!")
            return

        while not self.__app_quit:
            num_bytes = int(self.__sample_rate / 10) * self.__num_channels * BYTES_PER_SAMPLE
            buffer = sys.stdin.buffer.read(num_bytes)
            if buffer:
                self.__mic_device.write_frames(buffer)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required=True, help="Meeting URL")
    parser.add_argument(
        "-c", "--channels", type=int, default=NUM_CHANNELS, help="Number of channels"
    )
    parser.add_argument("-r", "--rate", type=int, default=SAMPLE_RATE, help="Sample rate")

    args = parser.parse_args()

    Daily.init()

    app = SendAudioApp(args.rate, args.channels)

    try:
        app.run(args.meeting)
    except KeyboardInterrupt:
        print("Ctrl-C detected. Exiting!")
    finally:
        app.leave()


if __name__ == "__main__":
    main()
