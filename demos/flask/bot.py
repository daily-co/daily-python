import io
import threading

from daily import *

from google.cloud import texttospeech

voice = texttospeech.VoiceSelectionParams(language_code="en-US", name="en-US-Studio-M")

audio_config = texttospeech.AudioConfig(
    audio_encoding=texttospeech.AudioEncoding.LINEAR16, speaking_rate=1.0, sample_rate_hertz=16000
)


class Bot:
    def __init__(self, name, microphone):
        self.__name = name

        self.__speech_client = texttospeech.TextToSpeechClient()

        self.__call_client = CallClient()

        self.__bot_error = None

        self.__start_event = threading.Event()
        self.__thread = threading.Thread(target=self.send_audio, args=[microphone])
        self.__thread.start()

    def on_joined(self, data, error):
        if error:
            print(f"Unable to join meeting: {error}")
            self.__bot_error = error
        self.__start_event.set()

    def run(self, meeting_url):
        self.__call_client.join(
            meeting_url,
            client_settings={
                "inputs": {
                    "camera": False,
                    "microphone": {"isEnabled": True, "settings": {"deviceId": "my-mic"}},
                }
            },
            completion=self.on_joined,
        )
        self.__thread.join()

    def leave(self):
        self.__call_client.leave()
        self.__call_client.release()

    def send_audio(self, microphone):
        self.__start_event.wait()

        if self.__bot_error:
            print(f"Unable to send audio!")
            return

        # NOTE: This is just an example. These sentences should probably come
        # from somewhere else.
        sentences = [
            "Hello. I hope you're doing well."
            "This is a bot written with Flask, Celery and Daily Python SDK."
            "Have a nice day!"
        ]

        for sentence in sentences:
            self.synthesize_sentence(microphone, sentence)

    def synthesize_sentence(self, microphone, sentence):
        synthesis_input = texttospeech.SynthesisInput(text=sentence.strip())

        response = self.__speech_client.synthesize_speech(
            input=synthesis_input, voice=voice, audio_config=audio_config
        )

        stream = io.BytesIO(response.audio_content)

        # Skip RIFF header
        stream.read(44)

        microphone.write_frames(stream.read())


#
# This is now a new process (because we created a Process in create_bot() in
# app.py), so it's safe to initialize Daily.init() as it will be executed just
# once in the new process.
#
# However, to pass information back to the main application we can't just return
# values from functions or update application global state, because processes
# are independent. But there are multiple alternatives:
#
# - Pipes and queues: https://billiard.readthedocs.io/en/latest/library/multiprocessing.html#pipes-and-queues
# - Redis Pub/Sub
# - More sophisticated messages queues: SQS, RabbitMQ, Kakfa
#


def start_bot(bot_name, meeting_url):
    Daily.init()

    microphone = Daily.create_microphone_device("my-mic", sample_rate=16000, channels=1)

    bot = Bot(bot_name, microphone)
    bot.run(meeting_url)
    bot.leave()
