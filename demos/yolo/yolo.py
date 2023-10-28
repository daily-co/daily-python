import argparse
import os
from ultralytics import YOLO
import time
from threading import Thread

from daily import EventHandler, CallClient, Daily
from datetime import datetime

class DailyYOLO(EventHandler):
    def __init__(
            self,
            url
        ):

        self.url = url
        self.model = YOLO('yolov5s.pt')
        self.configure_daily()

        while True:
            time.sleep(1)

    def configure_daily(self):
        Daily.init()
        self.client = CallClient(event_handler = self)
        self.camera = Daily.create_camera_device("camera", width = 1280, height = 720, color_format = "BGRA")
        self.client.join(self.url)
        self.camera_started = False

        self.client.update_inputs({
            "camera": {
                "isEnabled": True,
                "settings": {
                    "deviceId": "camera"
                }
            }
        })

    def on_participant_joined(self, participant):
        print(f"on_participant_joined: {participant}")

        if not participant["info"]["userName"]:
            return

        self.client.set_video_renderer(participant["id"],
                                         self.on_video_frame,
                                         color_format = "BGRA")
        

    def on_video_frame(self, participant_id, video_frame):
        self.camera.write_frame(video_frame.buffer)

        result = self.model(video_frame.buffer)
        result.print()

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="YOLO exmaple with Daily")
    parser.add_argument("-u", "--url", default="", type=str, help="URL of the Daily room")

    args = parser.parse_args()

    app = DailyYOLO(args.url)