#
# This demo will join a Daily meeting and record the meeting audio into a
# WAV. The WAV file will have a sample rate of 16000, 16-bit per sample and mono
# audio channel.
#
# Usage: python wav_audio_receive.py -m MEETING_URL -o FILE.wav
#

import argparse
import time
import wave

from daily import *

class ReceiveWavApp:
    def __init__(self, file_name):
        self.__speaker_device = Daily.create_speaker_device(
            "my-speaker",
            sample_rate = 16000,
            channels = 1
        )
        Daily.select_speaker_device("my-speaker")

        self.__wave = wave.open(file_name, "wb")
        self.__wave.setnchannels(1)
        self.__wave.setsampwidth(2) # 16-bit LINEAR PCM
        self.__wave.setframerate(16000)

        self.__client = CallClient()
        self.__client.update_subscription_profiles({
            "base": {
                "camera": "unsubscribed",
                "microphone": "subscribed"
            }
        })

    def join(self, meeting_url):
        self.__client.join(meeting_url)

    def leave(self):
        self.__client.leave()
        if self.__wave:
            self.__wave.close()

    def receive_audio(self):
        while True:
            buffer = self.__speaker_device.read_samples(800)
            if len(buffer) > 0:
                self.__wave.writeframesraw(buffer)
            time.sleep(0.05)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required = True, help = "Meeting URL")
    parser.add_argument("-o", "--output", required = False, help = "WAV output file")
    args = parser.parse_args()

    Daily.init()

    app = ReceiveWavApp(args.output)
    app.join(args.meeting)

    try :
        app.receive_audio()
    except KeyboardInterrupt:
        app.leave()

    # Let leave finish
    time.sleep(2)

if __name__ == '__main__':
    main()
