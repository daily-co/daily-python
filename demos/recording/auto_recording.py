#
# This demo will join a Daily meeting with a meeting token that automatically
# starts a cloud recording, and will send an image using a virtual camera device
# so the recording has some content.
#
# It needs a Daily API key to create the meeting token, read from the
# DAILY_API_KEY environment variable (a .env file is also loaded, see
# env.example).
#
# Usage: python3 auto_recording.py -m MEETING_URL
#

import argparse
import asyncio
import os
import threading
import time

import aiohttp
from daily import CallClient, Daily
from dotenv import load_dotenv
from PIL import Image
from pydantic import BaseModel, Field

# Load environment variables from .env file
load_dotenv(override=True)

SAMPLE_IMAGE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "sample.jpg")
FRAME_RATE = 30


class DailyStreamingOptions(BaseModel):
    """
    DailyStreamingOptions equivalent in Python.
    """

    width: int | None = Field(default=None, description="Width of the video stream.")
    height: int | None = Field(default=None, description="Height of the video stream.")
    fps: int | None = Field(default=None, description="Frames per second of the video stream.")
    videoBitrate: int | None = Field(default=None, description="Video bitrate in kbps.")
    audioBitrate: int | None = Field(default=None, description="Audio bitrate in kbps.")
    min_idle_timeout: int | None = Field(
        default=None, description="Minimum idle timeout in seconds."
    )
    max_duration: int | None = Field(
        default=None, description="Maximum duration of the streaming in seconds."
    )
    background_color: str | None = Field(
        default=None, description="Background color for the stream."
    )


class DailyMeetingTokenProperties(BaseModel):
    """Properties for configuring a Daily meeting token.

    We are only using here the properties needed to configure a Daily meeting starting cloud recording automatically.

    Refer to the Daily API documentation for more information:
    https://docs.daily.co/reference/rest-api/meeting-tokens/create-meeting-token#properties
    """

    exp: int | None = Field(
        default=None,
        description="Expiration time (unix timestamp in seconds). We strongly recommend setting this value for security. If not set, the token will not expire. Refer docs for more info.",
    )
    is_owner: bool | None = Field(
        default=None,
        description="If `true`, the token will grant owner privileges in the room. Defaults to `false`.",
    )
    start_cloud_recording: bool | None = Field(
        default=None,
        description="Start cloud recording when the user joins the room. This can be used to always record and archive meetings, for example in a customer support context.",
    )
    start_cloud_recording_opts: DailyStreamingOptions | None = Field(
        default=None,
        description="Start cloud recording options for configuring automatic cloud recording when the user joins the room.",
    )


class DailyMeetingTokenParams(BaseModel):
    """Parameters for creating a Daily meeting token.

    Refer to the Daily API documentation for more information:
    https://docs.daily.co/reference/rest-api/meeting-tokens/create-meeting-token#body-params
    """

    properties: DailyMeetingTokenProperties = Field(default_factory=DailyMeetingTokenProperties)


class DailyRESTHelper:
    """Helper class for interacting with Daily's REST API.

    Args:
        daily_api_key: Your Daily API key
        daily_api_url: Daily API base URL (e.g. "https://api.daily.co/v1")
        aiohttp_session: Async HTTP session for making requests
    """

    def __init__(
        self,
        *,
        daily_api_key: str,
        daily_api_url: str = "https://api.daily.co/v1",
        aiohttp_session: aiohttp.ClientSession,
    ):
        """Initialize the Daily REST helper."""
        self.daily_api_key = daily_api_key
        self.daily_api_url = daily_api_url
        self.aiohttp_session = aiohttp_session

    async def get_token(
        self,
        room_url: str,
        expiry_time: float = 60 * 60,
        owner: bool = True,
        params: DailyMeetingTokenParams | None = None,
    ) -> str:
        """Generate a meeting token for user to join a Daily room.

        Args:
            room_url: Daily room URL
            expiry_time: Token validity duration in seconds (default: 1 hour)
            owner: Whether token has owner privileges
            params: Parameters for creating a Daily meeting token

        Returns:
            str: Meeting token

        Raises:
            Exception: If token generation fails or room URL is missing
        """
        if not room_url:
            raise Exception(
                "No Daily room specified. You must specify a Daily room in order a token to be generated."
            )

        expiration: float = time.time() + expiry_time

        headers = {"Authorization": f"Bearer {self.daily_api_key}"}

        if params is None:
            params = DailyMeetingTokenParams(
                properties=DailyMeetingTokenProperties(is_owner=owner, exp=int(expiration))
            )
        else:
            params.properties.exp = int(expiration)
            params.properties.is_owner = owner

        json = params.model_dump(exclude_none=True)

        async with self.aiohttp_session.post(
            f"{self.daily_api_url}/meeting-tokens", headers=headers, json=json
        ) as r:
            if r.status != 200:
                text = await r.text()
                raise Exception(f"Failed to create meeting token (status: {r.status}): {text}")

            data = await r.json()

        return data["token"]


class AutoRecordingApp:
    def __init__(self, image_file, framerate):
        self.__image = Image.open(image_file)
        self.__framerate = framerate

        self.__camera = Daily.create_camera_device(
            "my-camera",
            width=self.__image.width,
            height=self.__image.height,
            color_format="RGB",
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

    def run(self, meeting_url, meeting_token):
        self.__client.join(
            meeting_url,
            meeting_token,
            client_settings={
                "inputs": {
                    "camera": {
                        "isEnabled": True,
                        "settings": {"deviceId": "my-camera"},
                    },
                    "microphone": False,
                }
            },
            completion=self.on_joined,
        )
        self.__thread.join()

    def leave(self):
        self.__app_quit = True
        # Unblock the sender thread in case we are leaving before the join
        # completed, otherwise it would wait forever.
        self.__start_event.set()
        self.__thread.join()
        self.__client.leave()
        self.__client.release()

    def send_image(self):
        self.__start_event.wait()

        if self.__app_error:
            print("Unable to send image!")
            return

        sleep_time = 1.0 / self.__framerate
        image_bytes = self.__image.tobytes()

        while not self.__app_quit:
            self.__camera.write_frame(image_bytes)
            time.sleep(sleep_time)


async def create_access_token(room_url: str) -> str:
    """Helper function to generate an access token.

    Returns:
        str: Access token

    Raises:
        Exception: If token generation fails
    """

    async with aiohttp.ClientSession() as aiohttp_session:
        daily_rest_helper = DailyRESTHelper(
            daily_api_key=os.getenv("DAILY_API_KEY", ""),
            daily_api_url=os.getenv("DAILY_API_URL", "https://api.daily.co/v1"),
            aiohttp_session=aiohttp_session,
        )

        token = await daily_rest_helper.get_token(
            room_url=room_url,
            params=DailyMeetingTokenParams(
                properties=DailyMeetingTokenProperties(
                    start_cloud_recording=True,
                    start_cloud_recording_opts=DailyStreamingOptions(
                        width=1920,
                        height=1080,
                        fps=30,
                        videoBitrate=4000,
                        audioBitrate=128,
                        max_duration=3600,
                    ),
                ),
            ),
        )
        if not token:
            raise Exception(f"Failed to get token for room: {room_url}")

    return token


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required=True, help="Meeting URL")
    args = parser.parse_args()

    # Only creating the token is async. The rest of the demo blocks, so we run
    # it outside of asyncio: this way a Ctrl-C is delivered directly as a
    # KeyboardInterrupt instead of going through asyncio's signal handling.
    meeting_token = asyncio.run(create_access_token(args.meeting))
    print(f"Meeting token: {meeting_token}")

    Daily.init()

    app = AutoRecordingApp(SAMPLE_IMAGE, FRAME_RATE)

    try:
        app.run(args.meeting, meeting_token)
    except KeyboardInterrupt:
        print("Ctrl-C detected. Exiting!")
    finally:
        app.leave()


if __name__ == "__main__":
    main()
