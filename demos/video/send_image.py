#
# This demo will join a Daily meeting and send a given image at the specified
# framerate using a virtual camera device.
#
# Usage: python3 send_image.py -m MEETING_URL -i IMAGE -f FRAME_RATE
#

import argparse
import time
import threading

from daily import *
from PIL import Image


class SendImageApp:
    def __init__(self, image_file, framerate):
        self.__image = Image.open(image_file)
        self.__framerate = framerate

        self.__camera = Daily.create_camera_device(
            "my-camera", width=self.__image.width, height=self.__image.height, color_format="RGB"
        )

        self.__client = CallClient()

        self.__client.update_subscription_profiles(
            {"base": {"camera": "unsubscribed", "microphone": "unsubscribed"}}
        )

        self.__app_quit = False
        self.__app_error = None

        self.__start_event = threading.Event()
        self.__thread = threading.Thread(target=self.send_image)
        self.__thread.start()

    def on_joined(self, data, error):
        if error:
            print(f"Unable to join meeting: {error}")
            self.__app_error = error
        self.__start_event.set()

    def run(self, meeting_url):
        self.__client.join(
            meeting_url,
            client_settings={
                "inputs": {
                    "camera": {"isEnabled": True, "settings": {"deviceId": "my-camera"}},
                    "microphone": False,
                }
            },
            completion=self.on_joined,
        )
        self.__thread.join()

    def leave(self):
        self.__app_quit = True
        self.__thread.join()
        self.__client.leave()
        self.__client.release()

    def send_image(self):
        self.__start_event.wait()

        if self.__app_error:
            print(f"Unable to send audio!")
            return

        sleep_time = 1.0 / self.__framerate
        image_bytes = self.__image.tobytes()

        while not self.__app_quit:
            self.__camera.write_frame(image_bytes)
            time.sleep(sleep_time)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required=True, help="Meeting URL")
    parser.add_argument("-i", "--image", required=True, help="Image to send")
    parser.add_argument("-f", "--framerate", type=int, required=True, help="Framerate")
    args = parser.parse_args()

    Daily.init()

    app = SendImageApp(args.image, args.framerate)

    try:
        app.run(args.meeting)
    except KeyboardInterrupt:
        print("Ctrl-C detected. Exiting!")
    finally:
        app.leave()


if __name__ == "__main__":
    main()
