#
# This demo will join a Daily meeting and it will capture audio from the default
# system microphone and send it to the meeting. It will also play the audio
# received from the meeting via the default system speaker.
#
# Usage: python3 record_and_play.py -m MEETING_URL
#

import argparse
import time

from daily import *

import pyaudio

SAMPLE_RATE=16000
NUM_CHANNELS=1
BYTES_PER_SAMPLE=2

class PyAudioApp:

    def __init__(self):
        self.__app_quit = False

        self.__virtual_mic = Daily.create_microphone_device(
            "my-mic",
            sample_rate = SAMPLE_RATE,
            channels = NUM_CHANNELS,
            non_blocking = True
        )
        self.__virtual_speaker = Daily.create_speaker_device(
            "my-speaker",
            sample_rate = SAMPLE_RATE,
            channels = NUM_CHANNELS,
            non_blocking = True
        )
        Daily.select_speaker_device("my-speaker")

        self.__pyaudio = pyaudio.PyAudio()
        self.__input_stream = self.__pyaudio.open(
            format=pyaudio.paInt16,
            channels=NUM_CHANNELS,
            rate=SAMPLE_RATE,
            input=True,
            stream_callback=self.on_input_stream
        )
        self.__output_stream = self.__pyaudio.open(
            format=pyaudio.paInt16,
            channels=NUM_CHANNELS,
            rate=SAMPLE_RATE,
            output=True,
            stream_callback=self.on_output_stream
        )

        self.__client = CallClient()

        self.__client.update_inputs({
            "camera": False,
            "microphone": {
                "isEnabled": True,
                "settings": {
                    "deviceId": "my-mic",
                    "customConstraints": {
                        "autoGainControl": { "exact": True },
                        "noiseSuppression": { "exact": True },
                        "echoCancellation": { "exact": True },
                    }
                }
            }
        }, completion = self.on_inputs_updated)

        self.__client.update_subscription_profiles({
            "base": {
                "camera": "unsubscribed",
                "microphone": "subscribed"
            }
        })

    def on_inputs_updated(self, inputs, error):
        if error:
            print(f"Unable to updated inputs: {error}")
            self.__app_quit = True

    def on_joined(self, data, error):
        if error:
            print(f"Unable to join meeting: {error}")
            self.__app_quit = True

    def run(self, meeting_url):
        self.__client.join(meeting_url, completion=self.on_joined)
        while not self.__app_quit:
            time.sleep(0.1)

    def leave(self):
        self.__app_quit = True
        self.__client.leave()

    def on_input_stream(self, in_data, frame_count, time_info, status):
        if self.__app_quit:
            return None, pyaudio.paAbort

        # If the microphone hasn't started yet `write_frames`this will return
        # 0. In that case, we just tell PyAudio to continue.
        self.__virtual_mic.write_frames(in_data)

        return None, pyaudio.paContinue

    def on_output_stream(self, ignore, frame_count, time_info, status):
        if self.__app_quit:
            return None, pyaudio.paAbort

        # If the speaker hasn't started yet `read_frames` will return 0. In that
        # case, we just create silence and pass it PyAudio and tell it to
        # continue.
        buffer = self.__virtual_speaker.read_frames(frame_count)
        if len(buffer) == 0:
            buffer = b'\x00' * frame_count * NUM_CHANNELS * BYTES_PER_SAMPLE

        return buffer, pyaudio.paContinue

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required = True, help = "Meeting URL")
    args = parser.parse_args()

    Daily.init()

    app = PyAudioApp()

    try :
        app.run(args.meeting)
    except KeyboardInterrupt:
        print("Ctrl-C detected. Exiting!")
    finally:
        app.leave()

    # Let leave finish
    time.sleep(2)

if __name__ == '__main__':
    main()
