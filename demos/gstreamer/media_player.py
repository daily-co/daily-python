#
# This demo will join a Daily meeting and send a given media file. The media
# file can be of any format supported by your local GStreamer.
#
# Usage: python3 media_player.py -m MEETING_URL -i FILE
#

from gi.repository import Gst, GstApp, GLib
import argparse
import time

from daily import *

import gi

gi.require_version('Gst', '1.0')
gi.require_version('GstApp', '1.0')

VIDEO_WIDTH = 1280
VIDEO_HEIGHT = 720
AUDIO_SAMPLE_RATE = 48000
AUDIO_CHANNELS = 2


class GstApp:

    def __init__(self, filename):
        self.__camera = Daily.create_camera_device("my-camera",
                                                   width=VIDEO_WIDTH,
                                                   height=VIDEO_HEIGHT,
                                                   color_format="I420")
        self.__microphone = Daily.create_microphone_device(
            "my-mic",
            sample_rate=AUDIO_SAMPLE_RATE,
            channels=AUDIO_CHANNELS,
            non_blocking=True
        )

        self.__client = CallClient()

        self.__client.update_subscription_profiles({
            "base": {
                "camera": "unsubscribed",
                "microphone": "unsubscribed"
            }
        })

        self.__player = Gst.Pipeline.new("player")

        source = Gst.ElementFactory.make("filesrc", None)
        source.set_property("location", filename)

        decodebin = Gst.ElementFactory.make("decodebin", None)
        decodebin.connect("pad-added", self.decodebin_callback)

        self.__player.add(source)
        self.__player.add(decodebin)
        source.link(decodebin)

        bus = self.__player.get_bus()
        bus.add_signal_watch()
        bus.connect("message", self.on_message)

        self.__loop = GLib.MainLoop()

    def on_joined(self, data, error):
        if error:
            print(f"Unable to join meeting: {error}")
        else:
            self.__player.set_state(Gst.State.PLAYING)

    def on_leave(self, ignore, error):
        if error:
            print(f"Error leaving meeting: {error}")
        self.__player.set_state(Gst.State.NULL)
        self.__loop.quit()

    def run(self, meeting_url):
        self.__client.join(meeting_url, client_settings={
            "inputs": {
                "camera": {
                    "isEnabled": True,
                    "settings": {
                        "deviceId": "my-camera"
                    }
                },
                "microphone": {
                    "isEnabled": True,
                    "settings": {
                        "deviceId": "my-mic"
                    }
                }
            },
            "publishing": {
                "camera": {
                    "isPublishing": True,
                    "sendSettings": {
                        "encodings": {
                            "low": {
                                "maxBitrate": 1000000,
                                "maxFramerate": 30.0,
                                "scaleResolutionDownBy": 1.0
                            }
                        }
                    }
                }
            }
        }, completion=self.on_joined)
        self.__loop.run()

    def leave(self):
        self.__client.leave(completion=self.on_leave)

    def on_message(self, bus, message):
        t = message.type
        if t == Gst.MessageType.EOS:
            self.leave()
        elif t == Gst.MessageType.ERROR:
            err, debug = message.parse_error()
            print(f"Error: {err} : {debug}")
            self.leave()
        return True

    def decodebin_callback(self, decodebin, pad):
        caps_string = pad.get_current_caps().to_string()
        if caps_string.startswith("audio"):
            self.decodebin_audio(pad)
        elif caps_string.startswith("video"):
            self.decodebin_video(pad)

    def decodebin_audio(self, pad):
        queue_audio = Gst.ElementFactory.make("queue", None)
        audioconvert = Gst.ElementFactory.make("audioconvert", None)
        audioresample = Gst.ElementFactory.make("audioresample", None)
        audiocapsfilter = Gst.ElementFactory.make("capsfilter", None)
        audiocaps = Gst.Caps.from_string(
            f"audio/x-raw,format=S16LE,rate={AUDIO_SAMPLE_RATE},channels={AUDIO_CHANNELS},layout=interleaved")
        audiocapsfilter.set_property("caps", audiocaps)
        appsink_audio = Gst.ElementFactory.make("appsink", None)
        appsink_audio.set_property("emit-signals", True)
        appsink_audio.connect("new-sample", self.appsink_audio_new_sample)

        self.__player.add(queue_audio)
        self.__player.add(audioconvert)
        self.__player.add(audioresample)
        self.__player.add(audiocapsfilter)
        self.__player.add(appsink_audio)
        queue_audio.sync_state_with_parent()
        audioconvert.sync_state_with_parent()
        audioresample.sync_state_with_parent()
        audiocapsfilter.sync_state_with_parent()
        appsink_audio.sync_state_with_parent()

        queue_audio.link(audioconvert)
        audioconvert.link(audioresample)
        audioresample.link(audiocapsfilter)
        audiocapsfilter.link(appsink_audio)

        queue_pad = queue_audio.get_static_pad("sink")
        pad.link(queue_pad)

    def decodebin_video(self, pad):
        queue_video = Gst.ElementFactory.make("queue", None)
        videoconvert = Gst.ElementFactory.make("videoconvert", None)
        videoscale = Gst.ElementFactory.make("videoscale", None)
        videocapsfilter = Gst.ElementFactory.make("capsfilter", None)
        videocaps = Gst.Caps.from_string(
            f"video/x-raw,format=I420,width={VIDEO_WIDTH},height={VIDEO_HEIGHT}")
        videocapsfilter.set_property("caps", videocaps)

        appsink_video = Gst.ElementFactory.make("appsink", None)
        appsink_video.set_property("emit-signals", True)
        appsink_video.connect("new-sample", self.appsink_video_new_sample)

        self.__player.add(queue_video)
        self.__player.add(videoconvert)
        self.__player.add(videoscale)
        self.__player.add(videocapsfilter)
        self.__player.add(appsink_video)
        queue_video.sync_state_with_parent()
        videoconvert.sync_state_with_parent()
        videoscale.sync_state_with_parent()
        videocapsfilter.sync_state_with_parent()
        appsink_video.sync_state_with_parent()

        queue_video.link(videoconvert)
        videoconvert.link(videoscale)
        videoscale.link(videocapsfilter)
        videocapsfilter.link(appsink_video)

        queue_pad = queue_video.get_static_pad("sink")
        pad.link(queue_pad)

    def appsink_audio_new_sample(self, appsink):
        buffer = appsink.pull_sample().get_buffer()
        (_, info) = buffer.map(Gst.MapFlags.READ)
        self.__microphone.write_frames(info.data)
        buffer.unmap(info)
        return Gst.FlowReturn.OK

    def appsink_video_new_sample(self, appsink):
        buffer = appsink.pull_sample().get_buffer()
        (_, info) = buffer.map(Gst.MapFlags.READ)
        self.__camera.write_frame(info.data)
        buffer.unmap(info)
        return Gst.FlowReturn.OK


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required=True, help="Meeting URL")
    parser.add_argument("-i", "--input", required=True, help="Video file")

    args = parser.parse_args()

    Gst.init(None)
    Daily.init()

    app = GstApp(args.input)

    try:
        app.run(args.meeting)
    except KeyboardInterrupt:
        print("Ctrl-C detected. Exiting!")
    finally:
        app.leave()

    # Let leave finish
    time.sleep(3)


if __name__ == '__main__':
    main()
