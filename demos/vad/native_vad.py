#
# This demo will join a Daily meeting and it will try to detect speech by
# analyzing incoming audio frames. There are a few arguments useful to improve
# speech detection during long sentences.
#
# Usage: python3 native_vad.py -m MEETING_URL
#

import argparse
import sys
import threading
import time

from enum import Enum

from daily import *


SAMPLE_RATE = 16000
NUM_CHANNELS = 1

SPEECH_THRESHOLD = 0.90
SPEECH_THRESHOLD_MS = 300
SILENCE_THRESHOLD_MS = 700
VAD_RESET_PERIOD_MS = 2000


class SpeechStatus(Enum):
    SPEAKING = 1
    NOT_SPEAKING = 2


class SpeechDetection:
    def __init__(self, speech_threshold_ms, silence_threshold_ms, sample_rate, num_channels):
        self.__speech_threshold = SPEECH_THRESHOLD
        self.__speech_threshold_ms = speech_threshold_ms
        self.__silence_threshold_ms = silence_threshold_ms

        self.__status = SpeechStatus.NOT_SPEAKING
        self.__started_speaking_time = 0
        self.__last_speaking_time = 0

        self.__vad = Daily.create_native_vad(
            reset_period_ms=VAD_RESET_PERIOD_MS, sample_rate=sample_rate, channels=num_channels
        )

    def analyze(self, buffer):
        confidence = self.__vad.analyze_frames(buffer)
        current_time_ms = time.time() * 1000

        if confidence > self.__speech_threshold:
            diff_ms = current_time_ms - self.__started_speaking_time

            if self.__status == SpeechStatus.NOT_SPEAKING:
                self.__started_speaking_time = current_time_ms

            if diff_ms > self.__speech_threshold_ms:
                self.__status = SpeechStatus.SPEAKING
                self.__last_speaking_time = current_time_ms
        else:
            diff_ms = current_time_ms - self.__last_speaking_time
            if diff_ms > self.__silence_threshold_ms:
                self.__status = SpeechStatus.NOT_SPEAKING

        if self.__status == SpeechStatus.SPEAKING:
            print("SPEAKING: " + str(confidence))
        else:
            print("NOT SPEAKING: " + str(confidence))


class NativeVadApp:
    def __init__(self, speech_threshold_ms, silence_threshold_ms, sample_rate, num_channels):
        self.__sample_rate = sample_rate

        self.__vad = SpeechDetection(
            speech_threshold_ms=speech_threshold_ms,
            silence_threshold_ms=silence_threshold_ms,
            sample_rate=sample_rate,
            num_channels=num_channels,
        )

        self.__speaker_device = Daily.create_speaker_device(
            "my-speaker", sample_rate=sample_rate, channels=num_channels
        )
        Daily.select_speaker_device("my-speaker")

        self.__client = CallClient()
        self.__client.update_subscription_profiles(
            {"base": {"camera": "unsubscribed", "microphone": "subscribed"}}
        )

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
            # Read 10ms worth of audio frames.
            buffer = self.__speaker_device.read_frames(int(self.__sample_rate / 100))
            if len(buffer) > 0:
                self.__vad.analyze(buffer)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required=True, help="Meeting URL")
    parser.add_argument(
        "-c", "--channels", type=int, default=NUM_CHANNELS, help="Number of channels"
    )
    parser.add_argument("-r", "--rate", type=int, default=SAMPLE_RATE, help="Sample rate")
    parser.add_argument(
        "-p", "--speech", type=int, default=SPEECH_THRESHOLD_MS, help="Speech threshold in ms"
    )
    parser.add_argument(
        "-s", "--silence", type=int, default=SILENCE_THRESHOLD_MS, help="Silence threshold in ms"
    )

    args = parser.parse_args()

    Daily.init()

    app = NativeVadApp(args.speech, args.silence, args.rate, args.channels)

    try:
        app.run(args.meeting)
    except KeyboardInterrupt:
        print("Ctrl-C detected. Exiting!", file=sys.stderr)
    finally:
        app.leave()


if __name__ == "__main__":
    main()
