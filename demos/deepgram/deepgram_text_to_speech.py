#
# This demo will join a Daily meeting and, given a text file with senteces (one
# per line), will translate text into audio using Deepgram's Text-To-Speech API
# and will send it into the meeting.
#
# The demo requires a Deepgram API key set in the DG_API_KEY environment variable.
#
# See https://developers.deepgram.com/docs/text-to-speech
#
# Usage: python3 deepgram_speech_to_text.py -m MEETING_URL -i FILE
#

import argparse
import os
import time

from daily import *
from deepgram import (
    DeepgramClient,
    SpeakOptions,
)

parser = argparse.ArgumentParser()
parser.add_argument("-m", "--meeting", required=True, help="Meeting URL")
parser.add_argument(
    "-i",
    "--input",
    required=True,
    help="File with sentences (one per line)")
args = parser.parse_args()

Daily.init()

# We create a virtual microphone device so we can read audio samples from the
# meeting.
microphone = Daily.create_microphone_device(
    "my-mic", sample_rate=16000, channels=1)

client = CallClient()

print()
print(f"Joining {args.meeting} ...")

# Join and tell our call client that we will be using our new virtual
# microphone.
client.join(args.meeting, client_settings={
    "inputs": {
        "microphone": {
            "isEnabled": True,
            "settings": {
                "deviceId": "my-mic"
            }
        }
    }
})

# Make sure we are joined. It would be better to use join() completion
# callback.
time.sleep(3)

sentences_file = open(args.input, "r")

deepgram = DeepgramClient(api_key=os.getenv("DG_API_KEY"))

speak_options = SpeakOptions(
    model="aura-asteria-en",
    encoding="linear16",
    sample_rate="16000",
    container="none"
)

print()

for sentence in sentences_file.readlines():
    print(f"Processing: {sentence.strip()}")
    print()

    speak_source = {
        "text": sentence.strip()
    }

    response = deepgram.speak.rest.v("1").stream_raw(speak_source, speak_options)

    # Send all the audio frames to the microphone.
    microphone.write_frames(response.read())

# Let everything finish
time.sleep(2)

client.leave()
client.release()
