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
# Usage: python gtk_app.py -m MEETING_URL -p PARTICIPANT_ID [-a] [-s]
#

import argparse
import sys
import wave

import cairo
import gi

gi.require_version("Gtk", "4.0")
from gi.repository import GLib, Gtk

from daily import *

class DailyGtkApp(Gtk.Application):
    def __init__(self, meeting_url, participant_id, save_audio, screen_share):
        super().__init__(application_id="co.daily.DailyGtkApp")

        self.__client = CallClient()
        self.__client.update_subscription_profiles({
            "base": {
                "microphone": "subscribed",
                "camera": "unsubscribed" if screen_share else "subscribed",
                "screenVideo": "subscribed" if screen_share else "unsubscribed",
            }
        })

        self.__width = 1280
        self.__height = 720

        self.__frame = None
        self.__frame_width = self.__width
        self.__frame_height = self.__height

        self.__black_frame = bytearray(self.__width * self.__height * 4)

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

        drawing_area = Gtk.DrawingArea()
        drawing_area.set_hexpand(True)
        drawing_area.set_vexpand(True)
        drawing_area.set_draw_func(self.drawing_area_draw, None)

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

        main_box.append(drawing_area)
        main_box.append(inputs_box)

        window.set_child(main_box)

        self.__button = button
        self.__drawing_area = drawing_area
        self.__meeting_entry = meeting_entry
        self.__participant_entry = participant_entry

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
                self.__wave.setsampwidth(2) # 16-bit LINEAR PCM
                self.__wave.setframerate(48000)

            self.join(meeting_url, participant_id)
            self.__button.set_label("Leave")

    def on_joined(self, data, error):
        if not error:
            self.__joined = True

    def on_left(self, data, error):
        self.__frame = None
        self.__drawing_area.queue_draw()
        self.__joined = False
        if self.__save_audio:
            self.__wave.close()

    def join(self, meeting_url, participant_id):
        if not meeting_url or not participant_id:
            return

        if self.__save_audio:
            self.__client.set_audio_renderer(participant_id, self.on_audio_data)

        self.__client.set_video_renderer(participant_id,
                                         self.on_video_frame,
                                         video_source = self.__video_source,
                                         color_format = "BGRA")

        self.__client.join(meeting_url, completion = self.on_joined)

    def leave(self):
        self.__client.leave(completion = self.on_left)

    def drawing_area_draw(self, area, context, w, h, data):
        if self.__joined and not self.__frame is None:
            image = bytearray(self.__frame.buffer)
        else:
            image = self.__black_frame

        width = self.__frame_width
        height = self.__frame_height

        stride = cairo.ImageSurface.format_stride_for_width (cairo.FORMAT_ARGB32, width)
        cairo_surface = cairo.ImageSurface.create_for_data(image, cairo.FORMAT_ARGB32, width, height, stride)

        width_ratio = float(self.__width) / float(width)
        height_ratio = float(self.__height) / float(height)
        scale_xy = min(height_ratio, width_ratio)

        context.scale(scale_xy, scale_xy)

        context.set_source_surface(cairo_surface)
        context.paint()

    def on_audio_data(self, participant_id, audio_data):
        self.__wave.writeframes(audio_data.audio_frames)

    def on_video_frame(self, participant_id, video_frame):
        self.__frame_width = video_frame.width
        self.__frame_height = video_frame.height
        self.__frame = video_frame
        self.__drawing_area.queue_draw()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", default = "", help = "Meeting URL")
    parser.add_argument("-p", "--participant", default = "", help = "Participant ID")
    parser.add_argument("-a", "--audio", default = False, action="store_true",
                        help = "Store participant audio in a file (participant-ID.wav)")
    parser.add_argument("-s", "--screen", default = False, action="store_true",
                        help = "Render screen share (if available) instead of camera")
    args = parser.parse_args()

    Daily.init()

    app = DailyGtkApp(args.meeting, args.participant, args.audio, args.screen)
    sys.exit(app.run())

if __name__ == '__main__':
    main()
