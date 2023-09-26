#
# This demo will join a Daily meeting and will receive and render video frames
# for a given participant ID.
#
# Usage: python gtk_app.py -m MEETING_URL -p PARTICIPANT_ID
#

import argparse
import queue
import time

import cairo
import gi

gi.require_version("Gtk", "4.0")
from gi.repository import GLib, Gtk

import numpy as np

from daily import *

class DailyGtkApp(Gtk.Application):
    def __init__(self, meeting_url, participant_id):
        super().__init__(application_id="co.daily.DailyGtkApp")

        self.__client = CallClient()
        self.__client.update_subscription_profiles({
            "base": {
                "camera": "subscribed",
                "microphone": "unsubscribed"
            }
        })

        self.__width = 1280
        self.__height = 720

        self.__frame = None
        self.__frame_width = self.__width
        self.__frame_height = self.__height

        self.__black_frame = np.zeros(self.__width * self.__height * 4)

        self.__joined = False
        self.__meeting_url = meeting_url
        self.__participant_id = participant_id

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
            self.join(meeting_url, participant_id)
            self.__button.set_label("Leave")

    def join(self, meeting_url, participant_id):
        if not meeting_url or not participant_id:
            return

        self.__client.set_video_renderer(participant_id,
                                         self.on_video_frame,
                                         color_format = "BGRA")
        self.__client.join(meeting_url)

        self.__joined = True

    def leave(self):
        self.__client.leave()
        self.__joined = False
        self.__drawing_area.queue_draw()
        # Let leave finish
        time.sleep(2)

    def drawing_area_draw(self, area, context, w, h, data):
        if self.__joined and not self.__frame is None:
            image = self.__frame
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

    def on_video_frame(self, participant_id, video_frame):
        self.__frame_width = video_frame.width
        self.__frame_height = video_frame.height
        self.__frame = np.frombuffer(video_frame.buffer, dtype=np.uint8).copy()
        self.__drawing_area.queue_draw()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", default = "", help = "Meeting URL")
    parser.add_argument("-p", "--participant", default = "", help = "Participant ID")
    args = parser.parse_args()

    Daily.init()

    app = DailyGtkApp(args.meeting, args.participant)
    exit_status = app.run()

if __name__ == '__main__':
    main()
