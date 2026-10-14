#
# This demo joins a Daily meeting and sends 1280x720 VP8 video with one of the
# camera send settings compared in the adaptive bitrate tests. It prints the
# sender's bandwidth estimate and send bitrate on every network stats update,
# so you can watch how fast each setting ramps up. Open the meeting in a
# browser to see the video it receives. It loops test_video.mp4 from this
# folder when present (or the file given with -v), otherwise random noise.
#
# Usage: python3 send_settings_test.py -m MEETING_URL -s SETTING [-v VIDEO] [-d SECONDS]
#

import argparse
import os
import threading
import time

import numpy as np
from daily import CallClient, Daily, EventHandler

WIDTH = 1280
HEIGHT = 720
FRAMERATE = 30

# The video used in the tests. It is kept out of git (see daily-python/.gitignore).
DEFAULT_VIDEO = os.path.join(os.path.dirname(os.path.abspath(__file__)), "test_video.mp4")


def default_abr():
    """A. Our defaults: 3 simulcast layers, adaptive bitrate (ABR) on."""
    return {"preferredCodec": "VP8", "allowAdaptiveLayers": True, "startBitrate": 1_000_000}


def abr_off():
    """B. The default 3 layers with ABR off."""
    return {"preferredCodec": "VP8", "allowAdaptiveLayers": False, "startBitrate": 1_000_000}


def single_layer():
    """C. ABR off, one full-resolution layer of up to 2 Mbps, starting at 2 Mbps."""
    return {
        "preferredCodec": "VP8",
        "allowAdaptiveLayers": False,
        "startBitrate": 2_000_000,
        "maxQuality": "low",
        "encodings": {
            "low": {"maxBitrate": 2_000_000, "maxFramerate": 30, "scaleResolutionDownBy": 1},
        },
    }


def abr_off_high_layer_2mbps():
    """D. ABR off, the default layers with the high layer raised to 2 Mbps, starting at 2 Mbps."""
    return {
        "preferredCodec": "VP8",
        "allowAdaptiveLayers": False,
        "startBitrate": 2_000_000,
        "maxQuality": "high",
        "encodings": {
            "low": {"maxBitrate": 90_000, "maxFramerate": 15, "scaleResolutionDownBy": 4},
            "medium": {"maxBitrate": 200_000, "maxFramerate": 15, "scaleResolutionDownBy": 2},
            "high": {"maxBitrate": 2_000_000, "maxFramerate": 30, "scaleResolutionDownBy": 1},
        },
    }


def abr_on_high_layer_2mbps():
    """E. ABR on, the default layers with the high layer raised to 2 Mbps, starting at 2 Mbps."""
    settings = abr_off_high_layer_2mbps()
    settings["allowAdaptiveLayers"] = True
    return settings


def abr_on_single_layer():
    """F. ABR on, one full-resolution layer of up to 2 Mbps, starting at 2 Mbps."""
    settings = single_layer()
    settings["allowAdaptiveLayers"] = True
    return settings


SETTINGS = {
    "default-abr": default_abr,
    "abr-off": abr_off,
    "single-layer": single_layer,
    "abr-off-high-2mbps": abr_off_high_layer_2mbps,
    "abr-on-high-2mbps": abr_on_high_layer_2mbps,
    "abr-on-single-layer": abr_on_single_layer,
}


class SendSettingsTestApp(EventHandler):
    def __init__(self, send_settings, video_file):
        self.__send_settings = send_settings

        self.__capture = None
        if video_file:
            import cv2

            self.__capture = cv2.VideoCapture(video_file)
            if not self.__capture.isOpened():
                raise RuntimeError(f"Unable to open video file {video_file}")

        self.__camera = Daily.create_camera_device(
            "my-camera", width=WIDTH, height=HEIGHT, color_format="RGB"
        )

        self.__client = CallClient(event_handler=self)
        self.__client.update_subscription_profiles(
            {"base": {"camera": "unsubscribed", "microphone": "unsubscribed"}}
        )

        self.__joined_at = None
        self.__last_stats_timestamp = None

        self.__app_quit = False
        self.__app_error = None

        self.__start_event = threading.Event()
        self.__thread = threading.Thread(target=self.send_video)
        self.__thread.start()

    def on_network_stats_updated(self, stats):
        latest = stats.get("stats", {}).get("latest", {})
        timestamp = latest.get("timestamp")
        if (
            self.__joined_at is None
            or timestamp is None
            or timestamp == self.__last_stats_timestamp
        ):
            return
        self.__last_stats_timestamp = timestamp

        elapsed = time.monotonic() - self.__joined_at
        estimate = latest.get("availableOutgoingBitrate") or 0
        sending = latest.get("videoSendBitsPerSecond") or 0
        print(
            f"{elapsed:6.1f} s  estimate {estimate / 1000:7.0f} kbps  sending {sending / 1000:7.0f} kbps"
        )

    def on_joined(self, data, error):
        if error:
            print(f"Unable to join meeting: {error}")
            self.__app_error = error
        else:
            self.__joined_at = time.monotonic()
            applied = self.__client.publishing()["camera"]["sendSettings"]
            print(f"Joined. Camera send settings: {applied}")
        self.__start_event.set()

    def run(self, meeting_url, duration):
        self.__client.join(
            meeting_url,
            client_settings={
                "inputs": {
                    "camera": {"isEnabled": True, "settings": {"deviceId": "my-camera"}},
                    "microphone": False,
                },
                "publishing": {"camera": {"sendSettings": self.__send_settings}},
            },
            completion=self.on_joined,
        )
        # Join with a timeout: on Windows, a join without one can't be
        # interrupted, so Ctrl-C would never be handled.
        while self.__thread.is_alive():
            joined_at = self.__joined_at
            if duration and joined_at is not None and time.monotonic() - joined_at > duration:
                break
            self.__thread.join(0.1)

    def leave(self):
        self.__app_quit = True
        self.__thread.join()
        self.__client.leave()
        self.__client.release()

    def next_frame(self):
        if self.__capture is None:
            # Random noise is hard to compress, so the encoder uses all the
            # bitrate it is given.
            return np.random.randint(0, 256, (HEIGHT, WIDTH, 3), dtype=np.uint8).tobytes()

        import cv2

        ok, frame = self.__capture.read()
        if not ok:
            self.__capture.set(cv2.CAP_PROP_POS_FRAMES, 0)
            ok, frame = self.__capture.read()
            if not ok:
                raise RuntimeError("Unable to read from the video file")
        frame = cv2.resize(frame, (WIDTH, HEIGHT))
        return cv2.cvtColor(frame, cv2.COLOR_BGR2RGB).tobytes()

    def send_video(self):
        self.__start_event.wait()

        if self.__app_error:
            print("Unable to send video!")
            return

        next_time = time.monotonic()
        while not self.__app_quit:
            self.__camera.write_frame(self.next_frame())
            next_time += 1.0 / FRAMERATE
            time.sleep(max(0.0, next_time - time.monotonic()))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required=True, help="Meeting URL")
    parser.add_argument(
        "-s",
        "--setting",
        required=True,
        choices=SETTINGS.keys(),
        help="Camera send settings to use",
    )
    parser.add_argument(
        "-v",
        "--video",
        help="Video file to loop (default: test_video.mp4 next to this script, if present, "
        "otherwise random noise)",
    )
    parser.add_argument(
        "-d",
        "--duration",
        type=float,
        help="Seconds to stay in the call (until Ctrl-C if not given)",
    )
    args = parser.parse_args()

    video = args.video
    if video is None and os.path.exists(DEFAULT_VIDEO):
        video = DEFAULT_VIDEO

    Daily.init()

    app = SendSettingsTestApp(SETTINGS[args.setting](), video)

    try:
        app.run(args.meeting, args.duration)
    except KeyboardInterrupt:
        print("Ctrl-C detected. Exiting!")
    finally:
        app.leave()


if __name__ == "__main__":
    main()
