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
import time

from daily import *
from deepgram import DeepgramClient

parser = argparse.ArgumentParser()
parser.add_argument("-m", "--meeting", required=True, help="Meeting URL")
parser.add_argument("-i", "--input", required=True, help="File with sentences (one per line)")
args = parser.parse_args()

Daily.init()

# We create a virtual microphone device so we can read audio samples from the
# meeting.
microphone = Daily.create_microphone_device("my-mic", sample_rate=16000, channels=1)

client = CallClient()

print()
print(f"Joining {args.meeting} ...")

# Join and tell our call client that we will be using our new virtual
# microphone.
client.join(
    args.meeting,
    client_settings={
        "inputs": {"microphone": {"isEnabled": True, "settings": {"deviceId": "my-mic"}}}
    },
)

# Make sure we are joined. It would be better to use join() completion
# callback.
time.sleep(3)

sentences_file = open(args.input, "r")

# Need DEEPGRAM_API_KEY environment variable.
deepgram = DeepgramClient()

print()

for sentence in sentences_file.readlines():
    print(f"Processing: {sentence.strip()}")
    print()

    response = deepgram.speak.v1.audio.generate(
        model="aura-2-asteria-en",
        encoding="linear16",
        container="none",
        sample_rate=16000,
        text=sentence.strip(),
    )

    # Send all the audio frames to the microphone.
    for data in response:
        microphone.write_frames(data)

# Let everything finish
time.sleep(2)

client.leave()
client.release()
