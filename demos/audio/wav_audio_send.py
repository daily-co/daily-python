#
# This demo will join a Daily meeting and send the audio from a WAV file into
# the meeting. The WAV file is required to have a sample rate of 16000, 16-bit
# per sample and mono audio channel.
#
# Usage: python3 wav_audio_send.py -m MEETING_URL -i FILE.wav
#

import argparse
import time
import threading
import wave

from daily import *

class SendWavApp:
    def __init__(self, input_file_name):
        self.__mic_device = Daily.create_microphone_device(
            "my-mic",
            sample_rate = 16000,
            channels = 1
        )

        self.__client = CallClient()

        self.__client.update_inputs({
            "camera": False,
            "microphone": {
                "isEnabled": True,
                "settings": {
                    "deviceId": "my-mic"
                }
            }
        }, completion = self.on_inputs_updated)

        self.__client.update_subscription_profiles({
            "base": {
                "camera": "unsubscribed",
                "microphone": "unsubscribed"
            }
        })

        self.__app_quit = False
        self.__app_error = None
        self.__app_joined = False
        self.__app_inputs_updated = False

        self.__start_event = threading.Event()
        self.__thread = threading.Thread(target = self.send_wav_file,
                                         args = [input_file_name]);
        self.__thread.start()

    def on_inputs_updated(self, inputs, error):
        if error:
            print(f"Unable to updated inputs: {error}")
            self.__app_error = error
        else:
            self.__app_inputs_updated = True
        self.maybe_start()

    def on_joined(self, data, error):
        if error:
            print(f"Unable to join meeting: {error}")
            self.__app_error = error
        else:
            self.__app_joined = True
        self.maybe_start()

    def run(self, meeting_url):
        self.__client.join(meeting_url, completion=self.on_joined)
        self.__thread.join()

    def leave(self):
        self.__app_quit = True
        self.__thread.join()
        self.__client.leave()

    def maybe_start(self):
        if self.__app_error:
            self.__start_event.set()

        if self.__app_inputs_updated and self.__app_joined:
            self.__start_event.set()

    def send_wav_file(self, file_name):
        self.__start_event.wait()

        if self.__app_error:
            print(f"Unable to send WAV file!")
            return

        wav = wave.open(file_name, "rb")

        sent_frames = 0
        total_frames = wav.getnframes()
        while not self.__app_quit and sent_frames < total_frames:
            # Read 100ms worth of audio frames.
            frames = wav.readframes(1600)
            if len(frames) > 0:
                self.__mic_device.write_frames(frames)
                sent_frames += 1600

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required = True, help = "Meeting URL")
    parser.add_argument("-i", "--input", required = True, help = "WAV input file")

    args = parser.parse_args()

    Daily.init()

    app = SendWavApp(args.input)

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
