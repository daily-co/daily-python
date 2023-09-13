Usage
====================================

The first thing we need to do before using the SDK is to initialize the
:class:`Daily` context.

.. code-block:: python

    from daily import *
    Daily.init()

See :func:`daily.Daily.init` for more details.

Creating a call client
--------------------------------------------------------

Most of the functionality of the SDK lies in the :class:`daily.CallClient`
class. A call client is used to join a meeting, handle meeting events,
sending/receiving audio and video, etc.

In order to create a client (after the SDK is initialized) we can simply do:

.. code-block:: python

    client = CallClient()

See :class:`daily.CallClient` for more details.

Joining a meeting
--------------------------------------------------------

The next step is to join a `Daily`_ meeting using a Daily meeting URL:

.. code-block:: python

    client.join("https://my.daily.co/meeting")

You might also need to pass a meeting token, for example, to join a private
room, or if you are the meeting owner. `Meeting tokens
<https://docs.daily.co/reference/rest-api/meeting-tokens>`_ provide access to
private rooms, and can pass some user-specific properties into the room.

.. code-block:: python

    client.join("https://my.daily.co/meeting", meeting_token = "MY_TOKEN")

See :func:`daily.CallClient.join` for more details.

Leaving a meeting
--------------------------------------------------------

It is important to leave the meeting in order to cleanup resources (e.g. network
connections).

.. code-block:: python

    client.leave()

See :func:`daily.CallClient.leave` for more details.

Setting the user name
--------------------------------------------------------

It is also possible to change the user name of our client. The user name is what
other participants might see as a description of you (e.g. Jane Doe).

.. code-block:: python

    client.set_user_name("Jane Doe")

See :func:`daily.CallClient.set_user_name` for more details.

Completion callbacks
--------------------------------------------------------

Some :class:`daily.CallClient` methods are asynchronous. In order to know when
those methods finish successfully or with an error, it's possible to optionally
register a callback at invocation time.

For example, below we will register a callback to know when a meeting join
succeeds.

.. code-block:: python

    def on_joined(join_data, error):
        if not error:
            print("We just joined the meeting!")

    client.join("https://my.daily.co/meeting", completion = on_joined)

Handling events
--------------------------------------------------------

During a meeting (or even before) events can be generated, for example when a
participant joins or leaves a meeting, when a participant changes their tracks
or when an app message is received.

To subscribe to events we need to subclass :class:`daily.EventHandler`. This can
be done by the main application class (if there's one) or by simply creating a
new class.

.. code-block:: python

    class MyApp(EventHandler):

We can then implement any of the event handlers defined by
:class:`daily.EventHandler` that we are interested in. For example, we could
handle the event when a participant joins by using
:func:`daily.EventHandler.on_participant_joined`:

.. code-block:: python

    class MyApp(EventHandler):

        def on_participant_joined(self, participant):
            print("New participant joined!")

Finally, we need to register the event handler when creating a
:class:`daily.CallClient`. For example:

.. code-block:: python

    class MyApp(EventHandler):

        def __init__(self):
            self.client = CallClient(event_handler = self)

Inputs and publishing settings
--------------------------------------------------------

Inputs and publishing settings specify if media can be sent and how it has to be
sent but, even if they are related, they are different.

**Inputs** deal with video and audio devices. With inputs we can update the
desired resolution of a camera or if the camera should be enabled or not. We can
also select our desired microphone.

With **publishing settings** we can specify if the video from the input camera
is being sent or not, and also the quality (e.g. bitrate) of the video we are
sending. Note however, that a camera can be enabled via inputs but it not be
published (i.e. sent).

See :func:`daily.CallClient.inputs` and :func:`daily.CallClient.publishing` for
more details.

Subscriptions and subscription profiles
--------------------------------------------------------

It is possible to receive both audio and video from all the participants or for
individual participants. This is done via the subscriptions and subscription
profiles functionality.

A **subscription** defines how we want to receive media. For example, at which
quality do we want to receive video.

A **subscription profile** gives a set of subscription media settings a
name. There is a predefined `base` subscription profile. Subscriptions profiles
can be assigned to participants and can be even updated for a specific
participant.

Updating subscription profiles
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

We can update the predefined `base` profile to subscribe to both camera and
microphone streams:

.. code-block:: python

    client.update_subscription_profiles({
        "base": {
            "camera": "subscribed",
            "microphone": "subscribed"
        }
    })

Unless otherwise specified (i.e. for each participant), this will apply to all
participants.

A more complicated example would be to define two profiles: `lower` and
`higher`.  The `lower` profile can be used to receive the lowest video quality
and the `higher` to receive the maximum video quality:

.. code-block:: python

    client.update_subscription_profiles({
        "lower" : {
            "camera": {
                "subscriptionState": "subscribed",
                "settings": {
                    "maxQuality": "low"
                }
            },
            "microphone": "unsubscribed"
        },
        "higher" : {
            "camera": {
                "subscriptionState": "subscribed",
                "settings": {
                    "maxQuality": "high"
                }
            },
            "microphone": "unsubscribed"
        }
   })

These profiles can then be assigned to particular participants. For example, the
participants that are shown as thumbnails can use the `lower` profile and the
active speaker can use the `higher` profile.

See :func:`daily.CallClient.update_subscription_profiles` for more details.

Assigning subscription profiles to participants
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Now that we have seen how subscription profiles work. Let's see how we can
assign a subscription profile to a participant:

.. code-block:: python

    client.update_subscriptions({
        "eb762a39-1850-410e-9b31-92d7b21d515c" : {
            "profile": "base",
            "media": {
                "camera": "subscribed",
            }
        }
    }, {
        "base": {
            "camera": "unsubscribed",
            "microphone": "unsubscribed"
        }
    })

In the example above we have updated the `base` profile by unsubscribing from
both camera and microphone. Then, we have assigned the `base` profile to
participant `eb762a39-1850-410e-9b31-92d7b21d515c` and subscribed to the camera
stream only for that participant.

See :func:`daily.CallClient.update_subscriptions` for more details.

Video and audio virtual devices
--------------------------------------------------------

A call client can specify virtual video and audio devices which can then be used
as simulated cameras, speakers or microphones.

Speakers and microphones
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

We can create speaker and microphone devices. Speakers are used to receive audio
from the meeting and microphones are used to send data to the
meeting. Currently, the audio from all the participants will be received mixed
into a speaker device.

In the following example we will create a new speaker device:

.. code-block:: python

    speaker = Daily.create_speaker_device("my-speaker", sample_rate = 16000, channels = 1)

and we will set it as our default speaker:

.. code-block:: python

    Daily.select_speaker_device("my-speaker")

After selecting the speaker device we will be able to receive audio from the
meeting by reading audio samples from the device.

Microphones are created in a similar way:

.. code-block:: python

    microphone = Daily.create_microphone_device("my-mic", sample_rate = 16000, channels = 1)

but they are differently via the call client input settings:

.. code-block:: python

    client.update_inputs({
        "microphone": {
            "isEnabled": True,
            "settings": {
                "deviceId": "my-mic"
            }
        }
    })

Once a microphone has been selected as an audio input (and we have joined a
meeting) we can send audio by writing audio samples to it. Those audio samples
will be sent as the call client participant audio.

See :func:`daily.Daily.create_speaker_device`,
:func:`daily.Daily.create_microphone_device`,
:func:`daily.Daily.select_speaker_device` and
:func:`daily.CallClient.update_inputs` for more details.

Multiple microphone devices
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Multiple microphones can be created, but only one can be active at the same
time. With a single call client this is easy to achieve, since we can simply set
it as the call client audio input as we saw before:

.. code-block:: python

    client.update_inputs({
        "microphone": {
            "isEnabled": True,
            "settings": {
                "deviceId": "my-mic"
            }
        }
    })

However, if multiple microphones are created and different call clients select
different microphones (all in the same application), we will certainly get
undesired behavior.

Sending and receiving raw media
--------------------------------------------------------

It is possible to receive video from a participant or send audio to the
meeting. In the following sections we will see how we can send and receive raw
media.

Receiving video from a participant
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Once we have created a call client we can register a callback to be called each
time a video frame is received from a specific participant.

.. code-block:: python

    client.set_video_renderer(PARTICIPANT_ID, on_video_frame)

where `on_video_frame` must be a function or a class method such as:

.. code-block:: python

    def on_video_frame(participant_id, video_frame):
        print(f"NEW FRAME FROM {participant_id}")

and where `video_frame` is a :class:`daily.VideoFrame`.

See :func:`daily.CallClient.set_video_renderer` for more details.

Receiving audio from a meeting
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Audio works a little bit differently than video. It is not possible to receive
audio for a single participant; instead, all the audio of the meeting will be
received.

In order to receive audio from the meeting, we need to create a speaker
device. To create a virtual speaker device, we need to initialize the SDK as
follows:

.. code-block:: python

    Daily.init(virtual_devices = True)

Then, we can create the device:

.. code-block:: python

    speaker = Daily.create_speaker_device("my-speaker", sample_rate = 16000, channels = 1)

and we need to select it before using it:

.. code-block:: python

    Daily.select_speaker_device("my-speaker")

Finally, after having joined a meeting, we can read samples from the speaker
(e.g. every 10ms):

.. code-block:: python

    while True:
        buffer = speaker.read_samples(160)
        time.sleep(0.01)

The audio format is 16-bit linear PCM.

See :func:`daily.VirtualSpeakerDevice.read_samples` for more details.

Sending audio to a meeting
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

To send audio into a meeting we need to create a microphone device and
initialize the SDK as before:

.. code-block:: python

    Daily.init(virtual_devices = True)

Then, create the microphone device:

.. code-block:: python

    microphone = Daily.create_microphone_device("my-mic", sample_rate = 16000, channels = 1)

The next step is to tell our client that we will be using our new microphone
device as the audio input:

.. code-block:: python

    client.update_inputs({
        "camera": False,
        "microphone": {
            "isEnabled": True,
            "settings": {
                "deviceId": "my-mic"
            }
        }
    })

Finally, after joining a meeting, we can write samples to the microphone device:

.. code-block:: python

    microphone.write_samples(samples)

The audio format is 16-bit linear PCM.

See :func:`daily.VirtualMicrophoneDevice.write_samples` for more details.


.. _Daily: https://daily.co
