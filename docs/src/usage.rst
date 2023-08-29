Usage
====================================

The first thing we need to do before using the SDK is to initialize the
:class:`Daily` context.

.. code-block:: python

    from daily import *
    Daily.init()


Creating a call client
--------------------------------------------------------

Most of the functionality of the SDK lies into the :class:`CallClient` class. A
call client is used to join a meeting, handle meeting events, sending/receiving
audio and video, etc.

In order to create a client (after the SDK is initialized) we can simply do:

.. code-block:: python

    client = CallClient()

Joining a meeting
--------------------------------------------------------

The next step is to join a `Daily`_ meeting using a Daily meeting URL:

.. code-block:: python

    client.join("https://my.daily.co/meeting")

If you are the meeting owner you will need a meeting token that can be also
specified during join:

.. code-block:: python

    client.join("https://my.daily.co/meeting", token = "MY_TOKEN")


Leaving a meeting
--------------------------------------------------------

It is important to leave the meeting in order to cleanup resources (e.g. network
connections).

.. code-block:: python

    client.leave()

.. _Daily: https://daily.co


Setting the user name
--------------------------------------------------------

After joining a meeting it is possible to change the user name of our
client. The user name is what other participants might see as a description of
you (e.g. Jane Doe).

.. code-block:: python

    client.set_user_name("Jane Doe")

Subscriptions and profiles
--------------------------------------------------------

It is possible to receive both audio and video from all the participants or for
individual participants. This is done via the subscriptions and subscription
profiles API.

A **subscription** defines how we want to receive media. For example, what
quality do we want to receive video.

A **subscription profile** gives a set of subscription settings a name. There is
a pre-defined `base` subscription profile. Subscriptions profiles can be
assigned to participants and can be even updated for a specific participant.

Subscriptions
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The following tables show the object fields used to define a subscription:

.. list-table:: **SubscriptionMediaSettings**
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "camera"
     - "subscribed" | "unsubscribed" | SubscriptionVideoSettings
   * - "microphone"
     - "subscribed" | "unsubscribed"
   * - "screenVideo"
     - "subscribed" | "unsubscribed" | SubscriptionVideoSettings
   * - "screenAudio"
     - "subscribed" | "unsubscribed"

.. list-table:: **SubscriptionVideoSettings**
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "subscriptionState"
     - "subscribed" | "unsubscribed"
   * - "settings"
     - ReceiveVideoSettings

.. list-table:: **ReceiveVideoSettings**
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "maxQuality"
     - "low" | "medium" | "high"


Subscription profiles
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The table below shows the object fields used to define a subscription profile:

.. list-table:: **SubscriptionSettings**
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - PROFILE_NAME
     - SubscriptionMediaSettings

Now we can subscribe to both camera and microphone streams from all
participants:

.. code-block:: python

    client.update_subscription_profiles({
        "base": {
            "camera": "subscribed",
            "microphone": "subscribed"
        }
    })

As we can see we have simply updated our `base` profile and tell it to subscribe
to `camera` and `microphone` form remote participants.

A more complicated example would be to define two profiles `lower` and `higher`.
The `lower` profile can be used to receive the lowest video quality and the
`higher` to receive the maximum video quality:

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

Assigning subscription profiles to participants
--------------------------------------------------------

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

The following tables show the object fields used to define a subscription:

.. list-table:: **ParticipantSubscription**
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - PARTICIPANT_ID
     - ParticipantSubscriptionSettings

.. list-table:: **ParticipantSubscriptionSettings**
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "profile"
     - PROFILE_NAME (e.g. "base")
   * - "media"
     - SubscriptionMediaSettings

Sending and receiving raw media
--------------------------------------------------------

It is possible to receive video from a participant or send audio to the
meeting. In the following sections we will see how we can send and receive raw
media.

Receiving video from a participant
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Once we have created a call client we can register a callback to be called each
time a video frame is received from a specific participant.

.. code-block:: python

    client.set_video_renderer(PARTICIPANT_ID, on_video_frame)

where `on_video_frame` must be a function or a class method such as:

.. code-block:: python

    def on_video_frame(participant_id, video_frame):
        print(f"NEW FRAME FROM {participant_id}")


Receiving audio from a meeting
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Audio works a little bit different than video. It is not possible to receive
audio for a single participant instead all the audio of the meeting will be
received.

In order to receive audio from the meeting we need to create a custom audio
device, think of it as a system speaker.

To create a custom audio device we need to initialize the SDK as follows:

.. code-block:: python

    Daily.init(custom_devices = True)

Then, we can create an audio device:

.. code-block:: python

    audio_device = Daily.create_custom_audio_device("my-audio-device")

It is possible to create multiple audio devices but only one can be selected at
a time:

.. code-block:: python

    Daily.select_custom_audio_device("my-audio-device")

Finally, after we have joined a meeting, we can read samples from the audio
device (e.g. every 10ms):

.. code-block:: python

    while True:
        buffer = audio_device.read_samples(160)
        time.sleep(0.01)

The audio format is 16-bit linear PCM.

Sending audio to a meeting
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

As we have seen in the previous section audio is a bit particular. In the case
of sending, think of a custom audio device as a system microphone.

To send audio to a meeting we also need to create a custom audio device and
therefore initialize the SDK as before:

.. code-block:: python

    Daily.init(custom_devices = True)

Then, create and select the audio device:

.. code-block:: python

    audio_device = Daily.create_custom_audio_device("my-audio-device")
    Daily.select_custom_audio_device("my-audio-device")

Once selected (and after joining a meeting) we can write samples to the audio
device (e.g. every 10ms):

.. code-block:: python

    audio_device.write_samples(samples)

The audio format is 16-bit linear PCM.
