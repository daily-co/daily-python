#
# This demo will join a Daily meeting and send raw audio received through the
# standard input. The audio format is required to have a sample rate of 16000,
# 16-bit per sample and mono audio channel.
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
import time
import threading

from daily import *

class SendAudioApp:
    def __init__(self):
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
        self.__thread = threading.Thread(target = self.send_raw_audio);
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

    def send_raw_audio(self):
        self.__start_event.wait()

        if self.__app_error:
            print(f"Unable to send audio!")
            return

        while not self.__app_quit:
            # 3200 bytes is 100ms (1600 * 2 bytes per sample)
            buffer = sys.stdin.buffer.read(3200)
            if buffer:
                self.__mic_device.write_frames(buffer)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required = True, help = "Meeting URL")
    args = parser.parse_args()

    Daily.init()

    app = SendAudioApp()

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
