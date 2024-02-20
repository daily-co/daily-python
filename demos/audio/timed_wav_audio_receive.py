#
# This demo will join a Daily meeting and record the meeting audio into a WAV
# for the given number of seconds (defaults to 10). The saved WAV file will have
# a sample rate of 16000, 16-bit per sample and mono audio channel.
#
# Usage: python3 timed_wav_audio_receive.py -m MEETING_URL -o FILE.wav [-s SECONDS]
#

import argparse
import sys
import time
import wave

from daily import *

quit = False

SAMPLE_RATE = 16000
BYTES_PER_SAMPLE = 2
NUM_CHANNELS = 1


def write_buffer_to_wav(filename, buffer):
    global quit

    with wave.open(filename, 'wb') as wav_file:
        wav_file.setnchannels(NUM_CHANNELS)
        wav_file.setsampwidth(BYTES_PER_SAMPLE)
        wav_file.setframerate(SAMPLE_RATE)
        wav_file.writeframes(buffer)

    print("done")

    quit = True


def on_join(speaker, output, seconds, error):
    # Since this is a non-blocking device, this function will behave
    # asynchronously. The completion callback will be called when the audio
    # frames have been read.
    if not error:
        speaker.read_frames(
            SAMPLE_RATE * seconds,
            completion=lambda buffer: write_buffer_to_wav(
                output,
                buffer))
        print("buffering", end="")


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
    args = parser.parse_args()

    Daily.init()

    speaker = Daily.create_speaker_device(
        "my-speaker",
        sample_rate=SAMPLE_RATE,
        channels=NUM_CHANNELS,
        non_blocking=True
    )
    Daily.select_speaker_device("my-speaker")

    client = CallClient()
    client.update_subscription_profiles({
        "base": {
            "camera": "unsubscribed",
            "microphone": "subscribed"
        }
    })
    client.join(
        args.meeting,
        completion=lambda data,
        error: on_join(
            speaker,
            args.output,
            args.seconds,
            error))

    try:
        while not quit:
            print(".", end="")
            sys.stdout.flush()
            time.sleep(0.2)
    except KeyboardInterrupt:
        print("Ctrl-C detected. Exiting!")
    finally:
        client.leave()
        client.release()


if __name__ == '__main__':
    main()
