import argparse
import os
import torch
import io
import time
from threading import Thread
from PIL import Image
import io

from daily import EventHandler, CallClient, Daily
from datetime import datetime

class DailyYOLO(EventHandler):
    def __init__(
            self,
            url
        ):

        self.url = url
        self.model = torch.hub.load('ultralytics/yolov5', 'yolov5s', pretrained=True)
        self.configure_daily()
        self.time = time.time()

        while True:
            time.sleep(1)

    def configure_daily(self):
        Daily.init()
        self.client = CallClient(event_handler = self)
        self.camera = Daily.create_camera_device("camera", width = 1280, height = 720, color_format="RGB")
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

        self.client.set_video_renderer(participant["id"],
                                         self.on_video_frame)
        

    def on_video_frame(self, participant_id, video_frame):
        if time.time() - self.time > 0.1:
            self.time = time.time()
            image = Image.frombytes("RGBA", (video_frame.width, video_frame.height), video_frame.buffer)
            result = self.model(image)

            pil = Image.fromarray(result.render()[0], mode="RGB").tobytes()

            self.camera.write_frame(pil)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="YOLO exmaple with Daily")
    parser.add_argument("-u", "--url", default="", type=str, help="URL of the Daily room")

    args = parser.parse_args()

    app = DailyYOLO(args.url)