#
# This demo will join a Daily meeting and will receive and render video frames
# for a given participant ID.
#
# If `-a` is specified, it will save a WAV file with the audio for only that
# participant and it will also reproduce it.
#
# If `-s` is specified, it will render the screen share (if available) otherwise
# it defaults to the participant camera.
#
# If `-c I420` is specified, it will receive I420 video frames (which needs Gtk
# 4.20 or newer to render) instead of BGRA.
#
# Usage: python gtk_app.py -m MEETING_URL -p PARTICIPANT_ID [-a] [-s] [-c BGRA|I420]
#

from daily import *
import argparse
import sys
import wave

import gi

# autopep8: off
gi.require_version("Gdk", "4.0")
gi.require_version("Gtk", "4.0")
from gi.repository import Gdk, GLib, Gtk  # pyright: ignore[reportAttributeAccessIssue]
# autopep8: on


class DailyGtkApp(Gtk.Application):
    def __init__(self, meeting_url, participant_id, save_audio, screen_share, color_format):
        super().__init__(application_id="co.daily.DailyGtkApp")

        self.__client = CallClient()
        self.__client.update_subscription_profiles(
            {
                "base": {
                    "microphone": "subscribed",
                    "camera": "unsubscribed" if screen_share else "subscribed",
                    "screenVideo": "subscribed" if screen_share else "unsubscribed",
                }
            }
        )

        self.__width = 1280
        self.__height = 720

        self.__color_format = color_format
        if color_format == "I420":
            # BT.601 with limited range, the same conversion used for BGRA frames.
            params = Gdk.CicpParams()
            params.set_color_primaries(1)
            params.set_transfer_function(13)
            params.set_matrix_coefficients(6)
            params.set_range(Gdk.CicpRange.NARROW)
            self.__yuv_color_state = params.build_color_state()

        self.__joined = False
        self.__meeting_url = meeting_url
        self.__participant_id = participant_id

        self.__save_audio = save_audio

        self.__video_source = "camera"
        if screen_share:
            self.__video_source = "screenVideo"

    def do_activate(self):
        window = Gtk.ApplicationWindow(application=self, title="daily-python Gtk demo")
        window.set_default_size(self.__width, self.__height)

        main_box = Gtk.Box(spacing=6, orientation=Gtk.Orientation.VERTICAL)
        inputs_box = Gtk.Box(spacing=6, orientation=Gtk.Orientation.HORIZONTAL)

        picture = Gtk.Picture()
        picture.set_hexpand(True)
        picture.set_vexpand(True)

        meeting_label = Gtk.Label(label="Meeting URL:")
        meeting_entry = Gtk.Entry()
        meeting_entry.set_hexpand(True)
        meeting_entry.set_text(self.__meeting_url)

        participant_label = Gtk.Label(label="Participant ID:")
        participant_entry = Gtk.Entry()
        participant_entry.set_hexpand(True)
        participant_entry.set_text(self.__participant_id)

        button = Gtk.Button(label="Join")
        button.connect("clicked", self.on_join_or_leave)

        inputs_box.append(meeting_label)
        inputs_box.append(meeting_entry)
        inputs_box.append(participant_label)
        inputs_box.append(participant_entry)
        inputs_box.append(button)

        main_box.append(picture)
        main_box.append(inputs_box)

        window.set_child(main_box)

        self.__button = button
        self.__meeting_entry = meeting_entry
        self.__participant_entry = participant_entry
        self.__picture = picture

        window.present()

    def on_join_or_leave(self, button):
        if self.__joined:
            self.leave()
            self.__button.set_label("Join")
        else:
            meeting_url = self.__meeting_entry.get_text()
            participant_id = self.__participant_entry.get_text()

            if self.__save_audio:
                self.__wave = wave.open(f"participant-{participant_id}.wav", "wb")
                self.__wave.setnchannels(1)
                self.__wave.setsampwidth(2)  # 16-bit LINEAR PCM
                self.__wave.setframerate(16000)

            self.join(meeting_url, participant_id)
            self.__button.set_label("Leave")

    def on_joined(self, data, error):
        if not error:
            self.__joined = True

    def on_left(self, error):
        self.__joined = False
        GLib.idle_add(self.show_frame, None)
        if self.__save_audio:
            self.__wave.close()

    def join(self, meeting_url, participant_id):
        if not meeting_url or not participant_id:
            return

        if self.__save_audio:
            self.__client.set_audio_renderer(participant_id, self.on_audio_data)

        self.__client.set_video_renderer(
            participant_id,
            self.on_video_frame,
            video_source=self.__video_source,
            color_format=self.__color_format,
        )

        self.__client.join(meeting_url, completion=self.on_joined)

    def leave(self):
        self.__client.leave(completion=self.on_left)

    def show_frame(self, video_frame):
        texture = None
        if self.__joined and video_frame is not None:
            texture = self.create_texture(video_frame)
        self.__picture.set_paintable(texture)
        return GLib.SOURCE_REMOVE

    def create_texture(self, video_frame):
        width = video_frame.width
        height = video_frame.height
        data = GLib.Bytes.new(video_frame.buffer)

        if self.__color_format == "BGRA":
            return Gdk.MemoryTexture.new(width, height, Gdk.MemoryFormat.B8G8R8A8, data, width * 4)

        # I420 frames hold the Y plane, then the U and V planes, which are half
        # the width and height of the frame (rounded up).
        chroma_width = (width + 1) // 2
        chroma_height = (height + 1) // 2

        # Gtk only renders I420 textures with an even width and height, so the
        # last column or row of other frames is left out.
        builder = Gdk.MemoryTextureBuilder()
        builder.set_bytes(data)
        builder.set_width(width - width % 2)
        builder.set_height(height - height % 2)
        builder.set_format(Gdk.MemoryFormat.G8_B8_R8_420)
        builder.set_color_state(self.__yuv_color_state)
        builder.set_stride_for_plane(0, width)
        builder.set_offset(0, 0)
        builder.set_stride_for_plane(1, chroma_width)
        builder.set_offset(1, width * height)
        builder.set_stride_for_plane(2, chroma_width)
        builder.set_offset(2, width * height + chroma_width * chroma_height)
        return builder.build()

    def on_audio_data(self, participant_id, audio_data, audio_source):
        self.__wave.writeframes(audio_data.audio_frames)

    def on_video_frame(self, participant_id, video_frame, video_source):
        # Frames arrive on a Daily thread, but Gtk widgets are only used from
        # the main loop.
        GLib.idle_add(self.show_frame, video_frame)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", default="", help="Meeting URL")
    parser.add_argument("-p", "--participant", default="", help="Participant ID")
    parser.add_argument(
        "-a",
        "--audio",
        default=False,
        action="store_true",
        help="Store participant audio in a file (participant-ID.wav)",
    )
    parser.add_argument(
        "-s",
        "--screen",
        default=False,
        action="store_true",
        help="Render screen share (if available) instead of camera",
    )
    parser.add_argument(
        "-c",
        "--color-format",
        default="BGRA",
        choices=["BGRA", "I420"],
        help="Color format of the received video frames",
    )
    args = parser.parse_args()

    if args.color_format == "I420" and Gtk.check_version(4, 20, 0):
        parser.error("I420 needs Gtk 4.20 or newer")

    Daily.init()

    app = DailyGtkApp(args.meeting, args.participant, args.audio, args.screen, args.color_format)
    sys.exit(app.run())


if __name__ == "__main__":
    main()
