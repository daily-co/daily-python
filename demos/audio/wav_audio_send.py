#
# This demo will join a Daily meeting and send the audio from a WAV file into
# the meeting. The WAV file is required to have a sample rate of 16000, 16-bit
# per sample and mono audio channel.
#
# Usage: python wav_audio_send.py -m MEETING_URL -i FILE.wav
#

import argparse
import time
import wave

from daily import *

class SendWavApp:
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
        })
        self.__client.update_subscription_profiles({
            "base": {
                "camera": "unsubscribed",
                "microphone": "unsubscribed"
            }
        })

    def join(self, meeting_url):
        self.__client.join(meeting_url)

    def leave(self):
        self.__client.leave()

    def send_wav_file(self, file_name):
        wav = wave.open(file_name, "rb")
        while True:
            sent_frames = 0
            total_frames = wav.getnframes()
            while sent_frames < total_frames:
                frames = wav.readframes(1600)
                frames_read = len(frames) / 2 # 16-bit linear PCM
                if frames_read > 0:
                    self.__mic_device.write_frames(frames)
                sent_frames += frames_read
            wav.rewind()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required = True, help = "Meeting URL")
    parser.add_argument("-i", "--input", required = True, help = "WAV input file")
    args = parser.parse_args()

    Daily.init()

    app = SendWavApp()
    app.join(args.meeting)

    # Here we could use join() completion callback or an EventHandler.
    time.sleep(2)

    try :
        app.send_wav_file(args.input)
    except KeyboardInterrupt:
        app.leave()

    # Let leave finish
    time.sleep(2)

if __name__ == '__main__':
    main()
