#
# This demo will join a Daily meeting and record the meeting audio into a WAV
# for the given number of seconds (defaults to 10). The saved WAV file will have
# a sample rate of 16000, 16-bit per sample and mono audio channel.
#
# Usage: python3 timed_wav_audio_receive.py -m MEETING_URL -o FILE.wav [-s SECONDS]
#

import argparse
import threading
import time
import sys
import wave

from daily import *

SAMPLE_RATE = 16000
BYTES_PER_SAMPLE = 2
NUM_CHANNELS = 1

class TimedReceiveWavApp(EventHandler):
    def __init__(self, output_file_name, sample_rate, num_channels, seconds):
        self.__output_file_name = output_file_name
        self.__seconds = seconds
        self.__sample_rate = sample_rate
        self.__num_channels = num_channels
        self.__speaker_device = Daily.create_speaker_device(
            "my-speaker",
            sample_rate=sample_rate,
            channels=num_channels,
            non_blocking=True
        )
        Daily.select_speaker_device("my-speaker")

        self.__client = CallClient(event_handler=self)
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

    def on_participant_updated(self, participant):
        if participant["info"]["isLocal"]:
            return
        if participant["media"]["microphone"]["state"] == "playable":
            self.__start_event.set()

    def run(self, meeting_url):
        self.__client.join(meeting_url)
        self.__thread.join()

    def leave(self):
        self.__thread.join()
        self.__client.leave()
        self.__client.release()

    def write_buffer_to_wav(self, buffer):
        with wave.open(self.__output_file_name, "wb") as wav_file:
            wav_file.setnchannels(self.__num_channels)
            wav_file.setsampwidth(BYTES_PER_SAMPLE)
            wav_file.setframerate(self.__sample_rate)
            wav_file.writeframes(buffer)
        self.__app_quit = True
        print("done")

    def receive_audio(self):
        print(f"waiting for a playable track")

        self.__start_event.wait()

        if self.__app_error:
            print("Unable to receive audio!")
            return

        print(f"buffering for {self.__seconds} seconds", end="")

        self.__speaker_device.read_frames(self.__sample_rate * self.__seconds,
            completion=lambda buffer: self.write_buffer_to_wav(buffer))

        while not self.__app_quit:
            print(".", end="")
            sys.stdout.flush()
            time.sleep(0.2)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required=True, help="Meeting URL")
    parser.add_argument(
        "-o",
        "--output",
        required=True,
        help="WAV output file")
    parser.add_argument(
        "-s",
        "--seconds",
        type=int,
        default=10,
        required=False,
        help="Number of seconds (default: 10)")
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
    args = parser.parse_args()

    Daily.init()

    app = TimedReceiveWavApp(args.output, args.rate, args.channels, args.seconds)

    try:
        app.run(args.meeting)
    except KeyboardInterrupt:
        print("Ctrl-C detected. Exiting!")
    finally:
        app.leave()

if __name__ == '__main__':
    main()
