#
# This demo will join a Daily meeting and it will capture audio from the default
# system microphone and send it to the meeting. It will also play the audio
# received from the meeting via the default system speaker.
#
# Usage: python3 record_and_play.py -m MEETING_URL
#

import argparse
import threading
import time

from daily import *

import pyaudio

SAMPLE_RATE = 16000
NUM_CHANNELS = 1


class PyAudioApp:

    def __init__(self, sample_rate, num_channels):
        self.__app_quit = False
        self.__num_channels = num_channels

        # We configure the microphone as non-blocking so we don't block PyAudio
        # when we write the frames.
        self.__virtual_mic = Daily.create_microphone_device(
            "my-mic",
            sample_rate=sample_rate,
            channels=num_channels,
            non_blocking=True
        )

        # In contrast, we configure the speaker as blocking. In this case,
        # PyAudio's output stream callback will wait until we get the data from
        # Daily's speaker.
        self.__virtual_speaker = Daily.create_speaker_device(
            "my-speaker",
            sample_rate=sample_rate,
            channels=num_channels,
            non_blocking=True
        )
        Daily.select_speaker_device("my-speaker")

        self.__pyaudio = pyaudio.PyAudio()
        self.__input_stream = self.__pyaudio.open(
            format=pyaudio.paInt16,
            channels=num_channels,
            rate=sample_rate,
            input=True,
            stream_callback=self.on_input_stream
        )
        self.__output_stream = self.__pyaudio.open(
            format=pyaudio.paInt16,
            channels=num_channels,
            rate=sample_rate,
            output=True
        )

        self.__client = CallClient()

        self.__client.update_subscription_profiles({
            "base": {
                "camera": "unsubscribed",
                "microphone": "subscribed"
            }
        })

        self.__thread = threading.Thread(target=self.send_audio_stream)
        self.__thread.start()

    def on_joined(self, data, error):
        if error:
            print(f"Unable to join meeting: {error}")
            self.__app_quit = True

    def run(self, meeting_url):
        self.__client.join(meeting_url, client_settings={
            "inputs": {
                "camera": False,
                "microphone": {
                    "isEnabled": True,
                    "settings": {
                        "deviceId": "my-mic",
                        "customConstraints": {
                            "autoGainControl": {"exact": True},
                            "noiseSuppression": {"exact": True},
                            "echoCancellation": {"exact": True},
                        }
                    }
                }
            },
            "publishing": {
                "microphone": {
                    "isPublishing": True,
                    "sendSettings": {
                        "channelConfig": "stereo" if self.__num_channels == 2 else "mono",
                    }
                }
            }
        }, completion=self.on_joined)
        self.__thread.join()

    def leave(self):
        self.__app_quit = True
        self.__client.leave()
        # This is not very pretty (taken from PyAudio docs).
        while self.__input_stream.is_active():
            time.sleep(0.1)
        self.__input_stream.close()
        self.__pyaudio.terminate()

    def on_input_stream(self, in_data, frame_count, time_info, status):
        if self.__app_quit:
            return None, pyaudio.paAbort

        # If the microphone hasn't started yet `write_frames` this will return
        # 0. In that case, we just tell PyAudio to continue.
        self.__virtual_mic.write_frames(in_data)

        return None, pyaudio.paContinue

    def on_speaker_frames(self, buffer):
        if not self.__app_quit:
            self.__output_stream.write(buffer)
            self.__virtual_speaker.read_frames(
                64, completion=self.on_speaker_frames)

    def send_audio_stream(self):
        self.__virtual_speaker.read_frames(
            64, completion=self.on_speaker_frames)
        while not self.__app_quit:
            time.sleep(0.1)
        self.__output_stream.close()


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
    args = parser.parse_args()

    Daily.init()

    app = PyAudioApp(args.rate, args.channels)

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
