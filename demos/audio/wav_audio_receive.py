#
# This demo will join a Daily meeting and record the meeting audio into a
# WAV. The WAV file will have a sample rate of 16000, 16-bit per sample and mono
# audio channel.
#
# Usage: python3 wav_audio_receive.py -m MEETING_URL -o FILE.wav
#

import argparse
import time
import threading
import wave

from daily import *


class ReceiveWavApp:
    def __init__(self, input_file_name):
        self.__speaker_device = Daily.create_speaker_device(
            "my-speaker",
            sample_rate=16000,
            channels=1
        )
        Daily.select_speaker_device("my-speaker")

        self.__wave = wave.open(input_file_name, "wb")
        self.__wave.setnchannels(1)
        self.__wave.setsampwidth(2)  # 16-bit LINEAR PCM
        self.__wave.setframerate(16000)

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
            buffer = self.__speaker_device.read_frames(1600)
            if len(buffer) > 0:
                self.__wave.writeframes(buffer)

        self.__wave.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required=True, help="Meeting URL")
    parser.add_argument(
        "-o",
        "--output",
        required=True,
        help="WAV output file")
    args = parser.parse_args()

    Daily.init()

    app = ReceiveWavApp(args.output)

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
