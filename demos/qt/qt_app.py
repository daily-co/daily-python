#
# This demo will join a Daily meeting and will receive and render video frames
# for a given participant ID.
#
# If `-a` is specified, it will save a WAV file with the audio for only that
# participant.
#
# If `-s` is specified, it will render the screen share (if available) otherwise
# it defaults to the participant camera.
#
# Usage: python qt_app.py -m MEETING_URL -p PARTICIPANT_ID [-a] [-s]
#

import argparse
import sys
import wave

from dataclasses import dataclass

from PySide6 import QtCore, QtGui, QtWidgets

import numpy as np

from daily import *

class DailyQtWidget(QtWidgets.QWidget):
    @dataclass
    class VideoFrameData:
        buffer: np.ndarray
        width: int
        height: int

    frame_signal = QtCore.Signal(VideoFrameData)

    def __init__(self, meeting_url, participant_id, save_audio, screen_share):
        super().__init__()

        self.__client = CallClient()
        self.__client.update_subscription_profiles({
            "base": {
                "microphone": "subscribed",
                "camera": "unsubscribed" if screen_share else "subscribed",
                "screenVideo": "subscribed" if screen_share else "unsubscribed",
            }
        })

        self.__frame_width = 1280
        self.__frame_height = 720

        self.frame_signal.connect(self.draw_image)

        self.__black_frame = QtGui.QPixmap(self.__frame_width, self.__frame_height)
        self.__black_frame.fill(QtGui.QColor('Black'))

        self.__joined = False
        self.__meeting_url = meeting_url
        self.__participant_id = participant_id

        self.__save_audio = save_audio
        if save_audio:
            self.__wave = wave.open(f"participant-{participant_id}.wav", "wb")
            self.__wave.setnchannels(1)
            self.__wave.setsampwidth(2) # 16-bit LINEAR PCM
            self.__wave.setframerate(48000)

        self.__video_source = "camera"
        if screen_share:
            self.__video_source = "screenVideo"

        self.setup_ui()

    def setup_ui(self):
        main_box = QtWidgets.QVBoxLayout(self)

        image_label = QtWidgets.QLabel()
        image_label.setPixmap(self.__black_frame)

        meeting_label = QtWidgets.QLabel("Meeting URL:")
        meeting_textedit = QtWidgets.QLineEdit()
        meeting_textedit.setText(self.__meeting_url)

        participant_label = QtWidgets.QLabel("Participant ID:")
        participant_textedit = QtWidgets.QLineEdit()
        participant_textedit.setText(self.__participant_id)

        button = QtWidgets.QPushButton("Join")
        button.clicked.connect(self.on_join_or_leave)

        inputs_box = QtWidgets.QHBoxLayout()
        inputs_box.addWidget(meeting_label)
        inputs_box.addWidget(meeting_textedit)
        inputs_box.addWidget(participant_label)
        inputs_box.addWidget(participant_textedit)
        inputs_box.addWidget(button)

        main_box.addWidget(image_label)
        main_box.addLayout(inputs_box)

        self.__button = button
        self.__image_label = image_label
        self.__meeting_textedit = meeting_textedit
        self.__participant_textedit = participant_textedit

    def on_join_or_leave(self):
        if self.__joined:
            self.leave()
            self.__button.setText("Join")
        else:
            meeting_url = self.__meeting_textedit.text()
            participant_id = self.__participant_textedit.text()
            self.join(meeting_url, participant_id)
            self.__button.setText("Leave")

    def on_joined(self, data, error):
        if not error:
            self.__joined = True

    def on_left(self, data, error):
        self.__image_label.setPixmap(self.__black_frame)
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

    def draw_image(self, frame_data):
        image = QtGui.QImage(frame_data.buffer, frame_data.width, frame_data.height,
                             frame_data.width * 4, QtGui.QImage.Format.Format_ARGB32)
        scaled = image.scaled(self.__frame_width, self.__frame_height, QtCore.Qt.AspectRatioMode.KeepAspectRatio)
        pixmap = QtGui.QPixmap.fromImage(scaled)
        self.__image_label.setPixmap(pixmap)

    def on_audio_data(self, participant_id, audio_data):
        self.__wave.writeframes(audio_data.audio_frames)

    def on_video_frame(self, participant_id, video_frame):
        data = np.frombuffer(video_frame.buffer, dtype=np.uint8).copy()
        frame_data = self.VideoFrameData(data, video_frame.width, video_frame.height)
        self.frame_signal.emit(frame_data)

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

    app = QtWidgets.QApplication([])

    widget = DailyQtWidget(args.meeting, args.participant, args.audio, args.screen)
    widget.resize(1280, 720)
    widget.show()

    sys.exit(app.exec())

if __name__ == '__main__':
    main()
