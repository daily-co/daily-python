import argparse
import queue
import time
import torch
import threading

from PIL import Image
from ultralytics import YOLO

from daily import *


class DailyYOLO(EventHandler):
    def __init__(self):
        self.__client = CallClient(event_handler=self)

        self.__model = YOLO("yolov8n.pt")
        self.__camera = None

        self.__time = time.time()

        self.__queue = queue.Queue()

        self.__app_quit = False

        self.__thread = threading.Thread(target=self.process_frames)
        self.__thread.start()

    def run(self, meeting_url):
        print(f"Connecting to {meeting_url}...")
        self.__client.join(meeting_url)
        print("Waiting for participants to join...")
        self.__thread.join()

    def leave(self):
        self.__app_quit = True
        self.__thread.join()
        self.__client.leave()
        self.__client.release()

    def on_participant_joined(self, participant):
        print(f"Participant {participant['id']} joined, analyzing frames...")
        self.__client.set_video_renderer(
            participant["id"], self.on_video_frame)

    def setup_camera(self, video_frame):
        if not self.__camera:
            self.__camera = Daily.create_camera_device(
                "camera",
                width=video_frame.width,
                height=video_frame.height,
                color_format="RGB")
            self.__client.update_inputs({
                "camera": {
                    "isEnabled": True,
                    "settings": {
                        "deviceId": "camera"
                    }
                }
            })

    def process_frames(self):
        while not self.__app_quit:
            video_frame = self.__queue.get()
            image = Image.frombytes(
                "RGBA", (video_frame.width, video_frame.height), video_frame.buffer)
            results = self.__model.track(image)

            pil = Image.fromarray(results[0].plot(), mode="RGB").tobytes()

            self.__camera.write_frame(pil)

    def on_video_frame(self, participant_id, video_frame):
        # Process ~15 frames per second (considering incoming frames at 30fps).
        if time.time() - self.__time > 0.05:
            self.__time = time.time()
            self.setup_camera(video_frame)
            self.__queue.put(video_frame)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required=True, help="Meeting URL")
    args = parser.parse_args()

    Daily.init()

    app = DailyYOLO()

    try:
        app.run(args.meeting)
    except KeyboardInterrupt:
        print("Ctrl-C detected. Exiting!")
    finally:
        app.leave()


if __name__ == '__main__':
    main()
