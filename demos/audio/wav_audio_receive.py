#
# This demo will start a Pipecat Cloud app, join the call, and record the
# meeting audio into a WAV.
#
#

import argparse
import os
import requests
import threading
import wave

from dotenv import load_dotenv
from daily import *


SAMPLE_RATE = 16000
NUM_CHANNELS = 1

load_dotenv(override=True)


class ReceiveWavApp:
    def __init__(self, input_file_name, sample_rate, num_channels):
        self.__sample_rate = sample_rate
        self.__speaker_device = Daily.create_speaker_device(
            "my-speaker", sample_rate=sample_rate, channels=num_channels, non_blocking=False
        )
        Daily.select_speaker_device("my-speaker")

        self.__wave = wave.open(input_file_name, "wb")
        self.__wave.setnchannels(num_channels)
        self.__wave.setsampwidth(2)  # 16-bit LINEAR PCM
        self.__wave.setframerate(sample_rate)

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

    def run(self, meeting_url, meeting_token=None):
        self.__client.join(meeting_url, meeting_token, completion=self.on_joined)
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
            # Read 100ms worth of audio frames.
            buffer = self.__speaker_device.read_frames(int(self.__sample_rate / 10))
            if len(buffer) > 0:
                self.__wave.writeframes(buffer)

        self.__wave.close()


def start_pipecat_app(api_key):
    response = requests.post(
        "https://api.pipecat.daily.co/v1/public/daily-python-virtual-speaker-test/start",
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        },
        json={
            "createDailyRoom": True,
            "transport": "daily",
            "dailyMeetingTokenProperties": {
                "is_owner": True
                # "enable_auto_recording": True
                # "start_cloud_recording": True
            },
            "dailyRoomProperties": {"enable_recording": "cloud"},
        },
    )
    response.raise_for_status()
    data = response.json()
    print(f"_____wav_audio_receive.py * data: {data}")
    return data["dailyRoom"], data.get("dailyToken")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-k", "--api-key", required=False, help="Pipecat API key")
    parser.add_argument(
        "-c", "--channels", type=int, default=NUM_CHANNELS, help="Number of channels"
    )
    parser.add_argument("-r", "--rate", type=int, default=SAMPLE_RATE, help="Sample rate")
    # parser.add_argument("-o", "--output", help="WAV output file")
    args = parser.parse_args()

    Daily.init()

    api_key = os.getenv("PIPECAT_API_KEY", args.api_key)
    room_url, token = start_pipecat_app(api_key)
    print(f"Started Pipecat app: {room_url}")

    room_name = room_url.rstrip("/").split("/")[-1]
    output = f"{room_name}.wav"
    app = ReceiveWavApp(output, args.rate, args.channels)

    try:
        # print(f"_____wav_audio_receive.py * token: {token}")
        app.run(room_url, token)
    except KeyboardInterrupt:
        print("Ctrl-C detected. Exiting!")
    finally:
        app.leave()


if __name__ == "__main__":
    main()
