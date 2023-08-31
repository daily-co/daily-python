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

If you are the meeting owner you will need a meeting token that can be also
specified during join:

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

After joining a meeting it is possible to change the user name of our
client. The user name is what other participants might see as a description of
you (e.g. Jane Doe).

.. code-block:: python

    client.set_user_name("Jane Doe")

See :func:`daily.CallClient.set_user_name` for more details.

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

Video and audio devices
--------------------------------------------------------

A call client can specify custom video and audio devices which can then be used
as simulated cameras, speakers or microphones.

Speakers and microphones
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

In the following example we will create a new :class:`daily.CustomAudioDevice`
(i.e. a simulated speaker and microphone):

.. code-block:: python

    audio_device = Daily.create_custom_audio_device("my-audio-device")

and we will set it as our call client microphone:

.. code-block:: python

    client.update_inputs({
        "microphone": {
            "isEnabled": True,
            "settings": {
                "deviceId": "my-audio-device"
            }
        }
    })

The created `audio_device` can now be used as a microphone and audio samples can
to be written into it (see :func:`daily.CustomAudioDevice.write_samples`). Those
audio samples will be sent as the call client participant audio.

As we just saw, we can select an audio input (i.e. a microphone) through
:func:`daily.CallClient.update_inputs`. However, it is important to understand
that audio inputs also act as audio output.

For example, we might want to write an application that only receives audio. In
this case, we still need to select the audio device through the `microphone`
input settings:

.. code-block:: python

    client.update_inputs({
        "microphone": {
            "isEnabled": False,
            "settings": {
                "deviceId": "my-audio-device"
            }
        }
    })

Note that in this case we have just disabled (`isEnabled` set to `False`) the
microphone, but `my-audio-device` will still be used as a speaker and,
therefore, audio sample can be read from it.

See :func:`daily.Daily.create_custom_audio_device` and
 :func:`daily.CallClient.update_inputs` for more details.

Multiple custom audio devices
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Multiple custom audio devices can be created, but only one can be active at the
same time. With a single call client this is easy to achieve, since we can
simply set it as the call client audio input as we saw before:

.. code-block:: python

    client.update_inputs({
        "microphone": {
            "isEnabled": True,
            "settings": {
                "deviceId": "my-audio-device"
            }
        }
    })

However, if multiple custom audio devices are created and different call clients
select different custom audio devices, we will certainly get undesired behavior.


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

In order to receive audio from the meeting, we need to create a
:class:`daily.CustomAudioDevice`. Think of it as a system speaker.

To create a custom audio device, we need to initialize the SDK as follows:

.. code-block:: python

    Daily.init(custom_devices = True)

Then, we can create an audio device:

.. code-block:: python

    audio_device = Daily.create_custom_audio_device("my-audio-device")

It is possible to create multiple audio devices, but all call clients will need
to configure the same device as an audio input.

Finally, after we have joined a meeting, we can read samples from the audio
device (e.g. every 10ms):

.. code-block:: python

    while True:
        buffer = audio_device.read_samples(160)
        time.sleep(0.01)

The audio format is 16-bit linear PCM.

See :func:`daily.CustomAudioDevice.read_samples` for more details.

Sending audio to a meeting
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

As we have seen in the previous sections, audio is a bit particular. In the case
of sending, think of a custom audio device as a system microphone.

To send audio into a meeting we also need to create a
:class:`daily.CustomAudioDevice` and initialize the SDK as before:

.. code-block:: python

    Daily.init(custom_devices = True)

Then, create and select the audio device:

.. code-block:: python

    audio_device = Daily.create_custom_audio_device("my-audio-device")
    Daily.select_custom_audio_device("my-audio-device")

The next step is to tell our client that we will be using our device
`my-audio-device` as the microphone. In order to do this, we will use the
:func:`daily.CallClient.inputs` method:

.. code-block:: python

    client.update_inputs({
        "camera": False,
        "microphone": {
            "isEnabled": True,
            "settings": {
                "deviceId": "my-audio-device"
            }
        }
    })

The above is necessary because otherwise our client will not know which audio
device to use as a microphone.

Finally, after joining a meeting, we can write samples to the audio device
(e.g. every 10ms):

.. code-block:: python

    audio_device.write_samples(samples)

The audio format is 16-bit linear PCM.

See :func:`daily.CustomAudioDevice.write_samples` for more details.

.. _Daily: https://daily.co
