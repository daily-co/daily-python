#
# This demo will join a Daily meeting and run YOLOv5 object detection on a single participant.

# See https://pypi.org/project/yolov5/ for more information on YOLOv5.
#
# Usage: python3 yolo.py -u URL
#

import argparse
import torch
import time
from PIL import Image

from daily import EventHandler, CallClient, Daily

class DailyYOLO(EventHandler):
    def __init__(
            self,
            url
        ):

        self.url = url
        self.model = torch.hub.load('ultralytics/yolov5', 'yolov5s', pretrained=True)
        self.camera = None
        self.configure_daily()
        self.time = time.time()

        while True:
            time.sleep(1)

    def configure_daily(self):
        Daily.init()
        self.client = CallClient(event_handler = self)
        self.client.join(self.url)

    def on_participant_joined(self, participant):
        print(f"on_participant_joined: {participant}")

        self.client.set_video_renderer(participant["id"],
                                         self.on_video_frame)
        

    def setup_camera(self, video_frame):
        if not self.camera:
            self.camera = Daily.create_camera_device("camera", width = video_frame.width, height = video_frame.height, color_format="RGB")
            self.client.update_inputs({
                "camera": {
                    "isEnabled": True,
                    "settings": {
                        "deviceId": "camera"
                    }
                }
            })

    def process_frame(self, video_frame):
        if time.time() - self.time > 0.1:
            self.time = time.time()
            image = Image.frombytes("RGBA", (video_frame.width, video_frame.height), video_frame.buffer)
            result = self.model(image)

            pil = Image.fromarray(result.render()[0], mode="RGB").tobytes()

            self.camera.write_frame(pil)

    def on_video_frame(self, participant_id, video_frame):
        self.setup_camera(video_frame)
        self.process_frame(video_frame)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="YOLO exmaple with Daily")
    parser.add_argument("-u", "--url", default="", type=str, help="URL of the Daily room")

    args = parser.parse_args()

    app = DailyYOLO(args.url)