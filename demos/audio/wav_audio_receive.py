#
# This demo will join a Daily meeting and record the meeting audio into a
# WAV.
#
# Usage: python3 wav_audio_receive.py -m MEETING_URL -o FILE.wav
#

import argparse
import time
import threading
import wave

from daily import *


SAMPLE_RATE = 16000
NUM_CHANNELS = 1


class ReceiveWavApp:
    def __init__(self, input_file_name, sample_rate, num_channels):
        self.__sample_rate = sample_rate
        self.__speaker_device = Daily.create_speaker_device(
            "my-speaker",
            sample_rate=sample_rate,
            channels=num_channels
        )
        Daily.select_speaker_device("my-speaker")

        self.__wave = wave.open(input_file_name, "wb")
        self.__wave.setnchannels(num_channels)
        self.__wave.setsampwidth(2)  # 16-bit LINEAR PCM
        self.__wave.setframerate(sample_rate)

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

    def receive_audio(self):
        self.__start_event.wait()

        if self.__app_error:
            print(f"Unable to receive audio!")
            return

        while not self.__app_quit:
            # Read 100ms worth of audio frames.
            buffer = self.__speaker_device.read_frames(
                int(self.__sample_rate / 10))
            if len(buffer) > 0:
                self.__wave.writeframes(buffer)

        self.__wave.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required=True, help="Meeting URL")
    parser.add_argument(
        "-c",
        "--channels",
        type=int,
        default=NUM_CHANNELS,
        help="Number of channels")
    parser.add_argument(
        "-r",
        "--rate",
        type=int,
        default=SAMPLE_RATE,
        help="Sample rate")
    parser.add_argument(
        "-o",
        "--output",
        required=True,
        help="WAV output file")
    args = parser.parse_args()

    Daily.init()

    app = ReceiveWavApp(args.output, args.rate, args.channels)

    try:
        app.run(args.meeting)
    except KeyboardInterrupt:
        print("Ctrl-C detected. Exiting!")
    finally:
        app.leave()

    # Let leave finish
    time.sleep(2)


if __name__ == '__main__':
    main()
