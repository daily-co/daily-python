Types
====================================

.. _AudioInputSettings:

AudioInputSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "deviceId"
     - string
   * - "customConstraints"
     - `MediaTrackConstraints <https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#properties>`_


.. _AudioPublishingSettings:

AudioPublishingSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "channelConfig"
     - "mono" | "stereo"
   * - "bitrate"
     - number


.. _AvailableDevices:

AvailableDevices
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "audio"
     - [ `MediaDeviceInfo`_ ]
   * - "camera"
     - [ `MediaDeviceInfo`_ ]
   * - "microphone"
     - [ `MediaDeviceInfo`_ ]
   * - "speaker"
     - [ `MediaDeviceInfo`_ ]


.. _CallClientError:

CallClientError
-----------------------------------

A string with an error message or *None*.


.. _CallClientJoinData:

CallClientJoinData
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "meetingSession"
     - `MeetingSession`_
   * - "participants"
     - `CallParticipants`_



.. _CallParticipants:

CallParticipants
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "local"
     - `Participant`_
   * - PARTICIPANT_UUID
     - `Participant`_


.. _CallState:

CallState
-----------------------------------

"initialized" | "joining" | "joined" | "leaving" | "left"

A new :class:`daily.CallClient` instance starts in the `initialized` state. As soon as
:func:`daily.CallClient.join` is invoked, it progresses to the `joining` state,
and when the client has joined the call, it progresses further to the `joined`
state. As soon as :func:`daily.CallClient.leave` is invoked, the state changes
to `leaving`, followed by `left` when this process has completed.


.. _CanAdminPermission:

CanAdminPermission
-----------------------------------

"participants" | "streaming" | "transcription"


.. _CanSendPermission:

CanSendPermission
-----------------------------------

"camera" | "microphone" | "screenVideo" | "screenAudio" | "customVideo" | "customAudio"


.. _CameraInputSettings:

CameraInputSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "isEnabled"
     - bool
   * - "settings"
     - `VideoInputSettings`_


.. _CameraPublishingSettings:

CameraPublishingSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "isPublishing"
     - bool
   * - "sendSettings"
     - `VideoPublishingSettings`_


.. _ClientSettings:

ClientSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "inputs"
     - `InputSettings`_
   * - "publishing"
     - `PublishingSettings`_


.. _ColorFormat:

ColorFormat
-----------------------------------

"ABGR" | "ARGB" | "BGRA" | "RGBA" | "RGB" | "I420"


.. _CustomVideoEncoding:

CustomVideoEncoding
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "quality"
     - "low" | "medium" | "high"
   * - "parameters"
     - `RTCRtpEncodingParameters <https://developer.mozilla.org/en-US/docs/Web/API/RTCRtpEncodingParameters>`_


.. _DialinEvent:

DialinEvent
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "participantId"
     - string
   * - "actionTraceId"
     - string
   * - "message"
     - string


.. _DialinConnectedEvent:

DialinConnectedEvent
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "actionTraceId"
     - string
   * - "displayName"
     - string
   * - "sipFrom"
     - string
   * - "sipHeaders"
     - Mapping[str, Any]

.. _DialinStoppedEvent:

DialinStoppedEvent
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "actionTraceId"
     - string
   * - "sipFrom"
     - string
   * - "sipHeaders"
     - Mapping[str, Any]

.. _DialoutEvent:

DialoutEvent
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "participantId"
     - string
   * - "actionTraceId"
     - string
   * - "message"
     - string


.. _DialoutSettings:

DialoutSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "sipUri"
     - string
   * - "phoneNumber"
     - string
   * - "callerId"
     - string


.. _IceConfig:

IceConfig
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "placement"
     - `IceConfigPlacement`_
   * - "iceServers"
     - [ `IceServer`_ ]


.. _IceConfigPlacement:

IceConfigPlacement
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Value
     - Description
   * - "replace"
     - Only the provided ICE servers are used
   * - "back"
     - The Daily-provided ICE servers are included first
   * - "front"
     - The provided ICE servers are included in the array first. This is the default behavior.


.. _IceServer:

IceServer
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "credential"
     - string
   * - "urls"
     - [ string ]
   * - "username"
     - string


.. _InputSettings:

InputSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "camera"
     - `CameraInputSettings`_
   * - "microphone"
     - `MicrophoneInputSettings`_


.. _LiveStreamState:

LiveStreamState
-----------------------------------

"connected" | "interrupted"


.. _LiveStreamStatus:

LiveStreamStatus
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "layout"
     - `StreamingLayout`_
   * - "startedBy"
     - string
   * - "streamId"
     - string


.. _LiveStreamUpdate:

LiveStreamUpdate
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "state"
     - `LiveStreamState`_
   * - "streamId"
     - string


.. _MediaDeviceInfo:

MediaDeviceInfo
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "deviceId"
     - string
   * - "groupId"
     - string
   * - "kind"
     - string
   * - "label"
     - string


.. _MeetingSession:

MeetingSession
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "id"
     - string


.. _MicrophoneInputSettings:

MicrophoneInputSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "isEnabled"
     - bool
   * - "settings"
     - `AudioInputSettings`_


.. _MicrophonePublishingSettings:

MicrophonePublishingSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "isPublishing"
     - bool
   * - "sendSettings"
     - "speech" | "music" | `AudioPublishingSettings`_


.. _NetworkDetailedStats:

NetworkDetailedStats
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "latest"
     - `NetworkLatestStats`_
   * - "worstVideoReceivePacketLoss"
     - number
   * - "worstVideoSendPacketLoss"
     - number


.. _NetworkLatestStats:

NetworkLatestStats
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "receiveBitsPerSecond"
     - number
   * - "sendBitsPerSecond"
     - number
   * - "timestamp"
     - number
   * - "totalRecvPacketLoss"
     - number
   * - "totalSendPacketLoss"
     - number
   * - "videoRecvBitsPerSecond"
     - number
   * - "videoRecvPacketLoss"
     - number
   * - "videoSendBitsPerSecond"
     - number
   * - "videoSendPacketLoss"
     - number


.. _NetworkStats:

NetworkStats
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "previousThreshold"
     - `NetworkThreshold`_
   * - "quality"
     - number
   * - "stats"
     - `NetworkDetailedStats`_
   * - "threshold"
     - `NetworkThreshold`_


.. _NetworkThreshold:

NetworkThreshold
-----------------------------------

"good" | "low" | "veryLow"


.. _Participant:

Participant
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "id"
     - string
   * - "info"
     - `ParticipantInfo`_
   * - "media"
     - `ParticipantMedia`_


.. _ParticipantCounts:

ParticipantCounts
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "hidden"
     - number
   * - "present"
     - number


.. _ParticipantInfo:

ParticipantInfo
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "isLocal"
     - bool
   * - "isOwner"
     - bool
   * - "joinedAt"
     - string
   * - "permissions"
     - `ParticipantPermissions`_
   * - "userId"
     - string
   * - "userName"
     - string


.. _ParticipantInputs:

ParticipantInputs
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "camera"
     - bool
   * - "microphone"
     - bool
   * - "screenShare"
     - bool


.. _ParticipantLeftReason:

ParticipantLeftReason
-----------------------------------

"leftCall" | "hidden"


.. _ParticipantMedia:

ParticipantMedia
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "camera"
     - `ParticipantMediaInfo`_
   * - "microphone"
     - `ParticipantMediaInfo`_
   * - "screenVideo"
     - `ParticipantMediaInfo`_
   * - "screenAudio"
     - `ParticipantMediaInfo`_


.. _ParticipantMediaInfo:

ParticipantMediaInfo
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "offReasons"
     - [ "user" | "bandwidth" | "sendPermission" | "remoteMute" ]
   * - "state"
     - "receivable" | "playable" | "loading" | "interrupted" | "blocked" | "off"
   * - "subscribed"
     - "subscribed" | "unsubscribed" | "staged"


.. _ParticipantPermissions:

ParticipantPermissions
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "hasPresence"
     - bool
   * - "canAdmin"
     - bool | [ `CanAdminPermission`_ ]
   * - "canSend"
     - bool | [ `CanSendPermission`_ ]


.. _ParticipantSubscriptions:

ParticipantSubscriptions
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - PARTICIPANT_UUID
     - `ParticipantSubscriptionSettings`_


.. _ParticipantSubscriptionSettings:

ParticipantSubscriptionSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "profile"
     - PROFILE_NAME (e.g. "base")
   * - "media"
     - `SubscriptionMediaSettings`_


.. _ParticipantUpdate:

ParticipantUpdate
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "permissions"
     - `ParticipantPermissions`_
   * - "inputsEnabled"
     - `ParticipantInputs`_


.. _PublishingSettings:

PublishingSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "camera"
     - `CameraPublishingSettings`_
   * - "microphone"
     - `MicrophonePublishingSettings`_


.. _ReceiveVideoSettings:

ReceiveVideoSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "maxQuality"
     - "low" | "medium" | "high"


.. _RecordingStatus:

RecordingStatus
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "layout"
     - `StreamingLayout`_
   * - "recordingId"
     - string
   * - "startedBy"
     - string
   * - "streamId"
     - string


.. _RemoteParticipantUpdates:

RemoteParticipantUpdates
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - PARTICIPANT_UUID
     - `ParticipantUpdate`_


.. _SipCallTransferSettings:

SipCallTransferSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "sessionId"
     - string
   * - "toEndPoint"
     - string


.. _StreamingAudioSettings:

StreamingAudioSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "bitrate"
     - number


.. _StreamingLayout:

StreamingLayout
-----------------------------------

For more details see the `layout object <https://docs.daily.co/reference/daily-js/instance-methods/start-recording#control-cloud-recording-layouts>`_.


.. _StreamingSettings:

StreamingSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "video"
     - `StreamingVideoSettings`_
   * - "audio"
     - `StreamingAudioSettings`_
   * - "maxDuration"
     - number
   * - "layout"
     - `StreamingLayout`_


.. _StreamingUpdateSettings:

StreamingUpdateSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "layout"
     - `StreamingLayout`_


.. _StreamingVideoSettings:

StreamingVideoSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "width"
     - number
   * - "height"
     - number
   * - "fps"
     - number
   * - "backgroundColor"
     - string (#rrggbb | #aarrggbb)
   * - "bitrate"
     - number


.. _SubscriptionMediaSettings:

SubscriptionMediaSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "camera"
     - "subscribed" | "unsubscribed" | `SubscriptionVideoSettings`_
   * - "microphone"
     - "subscribed" | "unsubscribed"
   * - "screenVideo"
     - "subscribed" | "unsubscribed" | `SubscriptionVideoSettings`_
   * - "screenAudio"
     - "subscribed" | "unsubscribed"


.. _SubscriptionProfileSettings:

SubscriptionProfileSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - PROFILE_NAME (e.g. "base")
     - `SubscriptionMediaSettings`_


.. _SubscriptionVideoSettings:

SubscriptionVideoSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "subscriptionState"
     - "subscribed" | "unsubscribed"
   * - "settings"
     - `ReceiveVideoSettings`_


.. _TranscriptionMessage:

TranscriptionMessage
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "participantId"
     - string
   * - "text"
     - string
   * - "timestamp"
     - string (ISO 8601)
   * - "rawResponse"
     - Mapping[str, Any] (includes Deepgram's response if `includeRawResponse` was enabled)

.. _TranscriptionSettings:

TranscriptionSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "language"
     - string (see Deepgram's `Language <https://developers.deepgram.com/docs/language>`_)
   * - "model"
     - string (see Deepgram's `Model <https://developers.deepgram.com/docs/model>`_)
   * - "tier"
     - string (this field is deprecated, use `model` instead)
   * - "profanity_filter"
     - bool (see Deepgram's `Profanity Filter <https://developers.deepgram.com/docs/profanity-filter>`_)
   * - "redact"
     - bool | list (see Deepgram's `Redaction <https://developers.deepgram.com/docs/redaction>`_)
   * - "punctuate"
     - bool (see Deepgram's `Punctuation <https://developers.deepgram.com/docs/punctuation>`_)
   * - "endpointing"
     - bool | number (see Deepgram's `Endpointing <https://developers.deepgram.com/docs/endpointing>`_)
   * - "extra"
     - Mapping[str, Any] (any additional Deepgram settings)
   * - "includeRawResponse"
     - bool (whether Deepgram's raw response should be included in all transcription messages)

.. _TranscriptionStatus:

TranscriptionStatus
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "language"
     - string (see Deepgram's `Language <https://developers.deepgram.com/docs/language>`_)
   * - "model"
     - string (see Deepgram's `Model <https://developers.deepgram.com/docs/model>`_)
   * - "tier"
     - string (this field is deprecated, use `model` instead)
   * - "startedBy"
     - string
   * - "instanceId"
     - string
   * - "transcriptId"
     - string

.. _TranscriptionUpdated:

TranscriptionUpdated
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "participants"
     - [ string ]
   * - "instanceId"
     - string
   * - "updatedBy"
     - string

.. _VideoInputSettings:

VideoInputSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "deviceId"
     - string (e.g. "my-video-camera")
   * - "width"
     - number
   * - "height"
     - number
   * - "frameRate"
     - number
   * - "facingMode"
     - "user" | "environment" | "left" | "right"
   * - "customConstraints"
     - `MediaTrackConstraints <https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#properties>`_

.. _VideoPublishingSettings:

VideoPublishingSettings
-----------------------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Key
     - Value
   * - "maxQuality"
     - "low" | "medium" | "high"
   * - "encodings"
     - "adaptiveHEVC" | [ `CustomVideoEncoding`_ ]
