# Changelog

All notable changes to the **daily-python** SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.19.0] - 2025-05-23

### Added

- Added a new `CustomAudioTrack`. This new track can be used as an additional
  custom audio track (i.e. with custom names) or as the main microphone track.

```python
audio_source = CustomAudioSource(...)

audio_track = CustomAudioTrack(audio_source)

client.join("YOUR_DAILY_ROOM", client_settings={
    "inputs": {
        "microphone": {
            "isEnabled": True,
            "settings": {
               "customTrack": {
                   "id": audio_track.id
               }
            }
        }
    }
})
```

- Added support for `CallClient.start_dialout()` new fields: `displayName`,
  `userId`, `video`, `codecs`, and `permissions`.

- `CallClient.set_audio_renderer()` can receive two additional arguments:
  `sample_rate` and `callback_interval_ms`. With `sample_rate` you can now
  specify the desired sample rate of the incoming audio data. With
  `callback_interval_ms` you can control how often the provided callback is
  called (with 10ms intervals).

### Changed

- `CallClient.add_custom_audio_track()` and
  `CallClient.update_custom_audio_track()` now receive a `CustomAudioTrack`
  instead of a `CustomAudioSource`.

- System certificates are now loaded on macOS, Linux and Windows platforms
  instead of the embedded Mozilla's root certificates.

### Fixed

- Fixed an issue that would cause a blocking virtual microphone to not send any
  audio in some cases.

- Handle meeting session ID changes which occur once the session has started

## [0.18.2] - 2025-05-07

### Fixed

- Fixed `daily-python` install issue on macOS < 15.0.

- Fixed `CallClient.remove_custom_track()` type hints.

## [0.18.1] - 2025-05-02

### Fixed

- Fixed an issue that would cause virtual microphones to not call the completion
  callbacks in some situations.

- Retrying multiple times to open the signalling channel during the join.

- Alternate websocket URI format to avoid connection issues potentially caused
  by ISPs.

## [0.18.0] - 2025-04-30

### Added

- It is now possible to create custom audio tracks with
  `CallClient.add_custom_audio_track()`. The custom audio tracks need a custom
  audio source which can be created with `CustomAudioSource()`. You can also
  update an existing custom track with a new audio source with
  `CallClient.update_custom_audio_track()` or remove and existing custom track
  with `CallClient.remove_custom_audio_track()`.

- Support the new `canReceive` permission, which involves:
  - Showing the proper track state (i.e. `"off"`, with `"receivePermission"`
    among the `"offReasons"`) when `canReceive` permission is lacking for a
    remote track.
  - Resuming receiving remote tracks when previously-revoked `canReceive`
    permissions have been restored.

### Changed

- Error messages "no subscription for consumer" and "Failed to close consumer"
  are now logged as debug messages since they can be caused by a remote
  participant simply disconnecting which is a valid and common case.

- Audio (`CallClient.set_audio_renderer()`) and video
  (`CallClient.set_video_renderer()`) renderers callbacks now receive the audio
  and video source as the last argument.

## [0.17.0] - 2025-03-27

### Fixed

- `daily-python` 0.16.x was compiled with glibc 2.35, which prevented installing
  it on systems with an older glibc version. This release goes back to glibc
  2.28.

## [0.16.1] - 2025-03-26

### Fixed

- Fixed an issue that was preventing new virtual speakers to get selected.

## [0.16.0] - 2025-03-25

### Added

- Added support for updating remote participants' `canReceive` permission via
  the `update_remote_participants()` method.

### Fixed

- Fixed an issue where the ICE servers configured by the user were not being
  applied when creating the PeerConnection.

## [0.15.0] - 2025-02-26

### Added

- Added support to automatically start a recording when joining a room if the
  `start_cloud_recording` token property is set to `True`.

### Fixed

- Fixed an issue where reconnection would fail if the meeting token was no
  longer valid.

## [0.14.2] - 2024-12-22

### Fixed

- Fixed type hints for `CallClient.send_app_message()`.

## [0.14.1] - 2024-12-22

### Changed

- Added `serialize_none` to `CallClient.send_app_message()` to allow for `None`
  values in object fields to be serialized to `null` or ignored otherwise.

### Fixed

- Fixed an issue in `CallClient.send_app_message()` that would not allow sending
  an object with `None` values.

## [0.14.0] - 2024-12-19

### Added

- Added support for sending dial-out DTMF tones via `CallClient.send_dtmf()`.

- Added support for SIP refer with `CallClient.sip_refer()`.

- Added support for SIP call transfers with `CallClient.sip_call_transfer()`.

- Added support for receiving ICE configuration from the SFU.

### Changed

- The field `participantId` from `DialoutEvent` and `DialinEvent` has been
  renamed to `sessionId`.

### Fixed

- Improved meeting move robustness by increasing the number of retries, to
  account for situations where the backend takes longer to complete the move.

## [0.13.0] - 2024-11-15

### Added

- Added support for `dialin-connected`, `dialin-stopped`, `dialin-warning`, and
  `dialin-error` events.

### Fixed

- Fixed a deadlock situation that would occur when reading from a virtual
  speaker before any remote participants joined the room and then trying to exit
  the application.

## [0.12.0] - 2024-10-23

### Added

- Added `CallClient.update_transcription()`. This allows choosing which
  participants should be transcribed.

- Added `EventHandler.on_transcription_updated()`.

- Daily Adaptive Bitrate (ABR) is now supported.
  (see https://www.daily.co/blog/introducing-daily-adaptive-bitrate/)

### Fixed

- Fixed a logging-related crash (stack overflow) that could occur when rapidly
  starting and stopping the SDK.

- Fixed an issue where missing fields in the domain/room permission config could
  cause a connection failure.

## [0.11.0] - 2024-09-16

### Added

- Added `callerId` field to `DialoutSettings`.

- Added `CallClient.start_live_stream()`, `CallClient.stop_live_stream()`,
  `CallClient.update_live_stream()`, `CallClient.add_live_streaming_endpoints()`
  and `CallClient.remove_live_streaming_endpoints()`.

- Added `EventHandler.on_live_stream_updated()`.

- Added support for specifying custom TURN servers via
  `CallClient.set_ice_config()`.

- Added support for specifying a proxy URL via `CallClient.set_proxy_url()`.

### Deprecated

- Transcription property `tier` has been deprecated. Use `model` instead.
  See https://developers.deepgram.com/docs/model

### Fixed

- Fixed an issue that caused app messages sent from the REST API to not be
  received correctly.

- Added missing dialin/dialout handlers in `daily.pyi`.

- `EventHandler.on_dialin_answered` should be
  `EventHandler.on_dialout_answered`.

- Fixed a crash caused by passing non-serializable data to
  `CallClient.send_app_message`.

- Fixed `daily.pyi` type completions.

## [0.10.1] - 2024-06-25

### Fixed

- Fixed an issue that would cause speaker/microphone completion callbacks to not
  be called if no other participant was in the call.

## [0.10.0] - 2024-06-20

### Added

- Added dial-out event `dialout-answered`.

### Other

- Added new example `demos/audio/async_wav_audio_send.py` that shows how to use
  `asyncio` with completion callbacks.

## [0.9.1] - 2024-05-29

### Fixed

- Fixed an issue where `VideoFrame.timestamp_us` was always 0.

## [0.9.0] - 2024-05-28

### Fixed

- Fixed a potential crash caused by audio/video renderers.

- Fixed an issue with audio/video renderers that was preventing renderers to be
  removed when participants leave.

### Performance

- CPU and memory performance improvements.

## [0.8.0] - 2024-05-23

### Added

- Added `dialin-ready` event.

- Added dial-out events `dialout-connected`, `dialout-stopped`, `dialout-error`
  and `dialout-warning`.

### Changed

- `CallClient.stop_dialout()` now takes the participant ID of the dial-out
  session we want to stop.

## [0.7.4] - 2024-04-16

### Fixed

- Fixed a crash caused by the following completion callbacks:
  `CallClient.update_inputs()`, `CallClient.update_publishing()`,
  `CallClient.update_subscriptions()`,
  `CallClient.update_subscription_profiles()`. The signature of the functions
  was wrongly documented as only a single `error` argument is passed.

- Fixed issue when calling `CallClient.update_publishing()` immediately after
  `CallClient.update_inputs()`.

## [0.7.3] - 2024-04-09

### Fixed

- Fixed an issue that could cause video and audio renderers not to work if they
  were registered after the media track was already being received.

## [0.7.2] - 2024-03-22

### Fixed

- Fixed missing milliseconds in client logs timestamps.

## [0.7.1] - 2024-03-08

### Fixed

- Fixed an issue that could cause join to fail if recording/transcription/live
  stream was started from the REST API.

## [0.7.0] - 2024-02-29

### Added

- Added initial support for low-level Voice Activity Detection (VAD).

```python
vad = Daily.create_native_vad(1000, 16000, 1)
confidence = vad.analyze_frames(audio_frames)
```

- Added `includeRawResponse` field to `TranscriptionSettings`. If true, all
  incoming `TranscriptionMessage` will include a new `rawResponse` field with
  Deepgram's raw data.

- Added new `CallClient.release()` function to allow freeing resources even in
  the event of `EventHandler` circular dependencies. It is common to have the
  following code:

```python
class MyClient(EventHandler):

  def __init__(self):
    self.call_client=CallClient(event_handler=self)

  ...
```

  If `MyClient` is a single application there shouldn't be any issues with
  freeing resources when the application ends. However, if we have an
  application that wants to create and release multiple `CallClient` instances
  the previous approach won't work with Python's garbage collection since
  there's a circular dependency. To solve this, we can now do:


```python
class MyClient(EventHandler):

  def __init__(self):
    self.call_client=CallClient(event_handler=self)

  def leave(self):
    self.call_client.leave()
    self.call_client.release()

  ...
```

  The new `CallClient.release()` function also blocks until all previous
  asynchronous operations have completed, so it's another convenient way to
  know, for example, when `CallClient.leave()` finishes.

### Changed

- ⚠️ Breaking change ⚠️: Completion callbacks now receive only the necessary
  arguments. For example, before `CallClient.leave(completion=...)` completion
  callback would receive `(None, Error | None)` arguments when it should only
  receive `(Error | None)`.

  This is the list of functions with completion callbacks that have been
  affected:
  `CallClient.leave()`, `CallClient.update_remote_participants()`,
  `CallClient.eject_remote_participants()`, `CallClient.update_permissions()`,
  `CallClient.start_recording()`, `CallClient.stop_recording()`,
  `CallClient.update_recording()`, `CallClient.start_transcription()`,
  `CallClient.stop_transcription()`, `CallClient.start_dialout()`,
  `CallClient.stop_dialout()`, `CallClient.send_app_message()`,
  `CallClient.send_prebuilt_chat_message()`.

  If you use any of the completion callbacks from one of the functions listed
  above, you simply need to remove the first argument from your callable.

### Other

- Updated demos to use the new `CallClient.release()` function.

## [0.6.4] - 2024-02-28

### Fixed

- Fixed an issue that would not allow join to succeed if a React Native client
  was already in the room.

## [0.6.3] - 2024-02-22

### Added

- Add support for `audio-only` recording layout preset.

## [0.6.2] - 2024-02-14

### Fixed

- Virtual microphones now always send audio (silence) if the user doesn't
  provide audio frames.

- Fix deadlock when registering completion callbacks inside callbacks.

- Enable Opus FEC to improve audio with network packet loss.

- Fixed multiple issues which could cause a deadlock during network
  reconnection.

- Ensure that `CallClient.update_inputs()` continues to be usable while the
  network is down.

- Fixed a crash which could occur if the network connection drops soon after
  joining.

### Other

- Simplied demos by using `client_settings` parameter in `CallClient.join()`
  instead of a separate `CallClient.update_inputs()` call.

- Updated `pyaudio` demo to only use non-blocking virtual devices.

## [0.6.1] - 2024-01-22

### Fixed

- Disable Opus DTX (discontinuous transmission). This improves audio quality and
  fixes issues on recordings.

## [0.6.0] - 2024-01-18

### Added

- Added `punctuate` and `endpointing` fields to `TranscriptionSettings`.

- Added dialout support with `CallClient.start_dialout()` and
  `CallClient.stop_dialout()`.

- Added completion callbacks to `VirtualMicrophone.write_frames()` and
  `VirtualSpeaker.read_frames()`. This change makes virtual devices
  completely asynchronous if they are created with `non_blocking` set to
  `True`.

### Changed

- Renamed `session_id` field to `participantId` in `TranscriptionMessage`.

### Removed

- Removed `is_final`, `user_id` and `user_name` fields from
  `TranscriptionMessage`.

### Fixed

- Room deletion messages from the server are now properly handled.

- `CallClient.send_app_message(None)` now properly triggers a `ValueError`
  exception.

- If an invalid participant ID is passed to `CallClient.send_app_message()` it
  will now trigger a `ValueError` exception.

- Fixed an issue that would cause audio crackling and popping when using
  non-blocking devices.

- Fixed support for different audio sample rates and number of channels, other
  than 16000 and 1 channel.

- Don't quote the participant ID when passing the string to video/audio renderer
  callbacks.

- Fixed a potential crash on shutdown when using a virtual camera device.

- Emit `transcription-started` event if transcription is already started when
  joining the room.

### Other

- Added GStreamer media player demo.

## [0.5.4] - 2023-12-08

### Fixed

- Fixed another issue that could cause `CallClient.join()` to fail if another
  Daily web client was also joining at the same time.

## [0.5.3] - 2023-12-08

### Fixed

- Fixed an issue that could cause `CallClient.join()` to fail if another Daily
  web client was also joining at the same time.

## [0.5.2] - 2023-12-05

### Fixed

- Disabled echo cancellation, noise suppression and auto gain control by default
  to match the previous library behavior.

## [0.5.1] - 2023-11-30

### Fixed

- Fixed a crash when passing audio frames to `VirtualMicrophone.write_frames()`
  that require padding (i.e. non-multiple of 10ms worth of audio frames).

## [0.5.0] - 2023-11-30

### Added

- Support for non-blocking virtual audio devices. This allows integration with
  hardware devices (e.g. via PyAudio).

- Echo cancellation, noise suppression and auto gain control can now be enabled
  for virtual microphones via custom constraints.

- It is now possible to pass additional Deepgram settings to
  `start_transcription()` using the new `extra` field.

### Changed

- Transcription defaults have been removed in favor of Deepgram's defaults. This
  allows to simply specify `{"model": "nova-2"}`.

- Transcription `redact` can now also be a list of strings as supported by
  Deepgram (e.g. `["pci"]`).

### Fixed

- Fixed an issue on user leave (manual or by the server) that would prevent the
  user to rejoin.

### Other

- New demos to show how to integrate with PyAudio, how to send images and other
  improvements in existing demos.

## [0.4.0] - 2023-11-09

### Added

- Added support for capturing individual participant audio tracks.

- Added support for ejecting participants.

- Support python >= 3.7 and, on Linux, glibc >= 2.28.

### Changed

- Transcription defaults have been removed in favor of Deepgram's defaults. This
  allows to simply specify `{"model": "nova-2"}`.

- Transcription redact can now also be a list of strings as supported by
  Deepgram (e.g. `["pci"]`).

### Fixed

- Fixed a deadlock that would not allow receiving multiple simultaneous video
  renderers.

- Fixed a deadlock when a screen share was stopped.

- Fixed an issue where setting the user name could not be reflected
  automatically when requesting participants list.

- Fixed an issue that could cause joins/reconnects to not complete successfully.

- Fixed a sporadic crash that could occur when handling media streams.

### Performance

- Improved general video renderer performance.

- Improved media subscriptions stability and performance.

### Other

- Added Qt demo (similar to the existing Gtk demo).

- Qt and Gtk demos can now save the selected participant audio into a WAV file
  and can also render screen share.

## [0.3.1] - 2023-10-25

### Fixed

- Fixed an issue that could cause daily-python clients to join a session in a
  different region.

- Fixed a dead-lock that could occur when a `CallClient` is destroyed.

## [0.3.0] - 2023-10-23

### Added

- Support for sending chat messages to Daily Prebuilt
  (`CallClient.send_prebuilt_chat_message()`).

- Added Python type hints (helpful for editor completions).

- Support for Python 3.8.

### Changed

- `EventHandler.on_transcription_stopped` can now tell if transcription was
  stopped by a user or because of an error occurred.

### Removed

- Removed `detect_language` from `TranscriptionSettings`.

### Fixed

- Improved response time of `CallClient` getter functions.

- Improved low-latency performace of virtual audio devices.

- Fixed potential crash after `CallClient.leave()`.

- Improved internal safeness of participant video renderers.

- Fixed a `VirtualMicrophoneDevice` memory leak.

- Properly trigger a transcription error event if transcription can't start.

### Other

- Demos have been updated to show more real live code.

## [0.2.0] - 2023-10-03

### Added

- Support for start/stop recordings.

- Support for start/stop transcriptions and receive transcriptions messages.

### Changed

- `VirtualSpeakerDevice.read_frames()` has been improved and doesn't require the
  user to add sleeps. Therefore, it is now possible to read, for example, 10
  seconds of audio in a single call. Since the timings are now controlled
  internally, this minimizes any potential audio issues.

  The following old code:

```python
SAMPLE_RATE = 16000
READ_INTERVAL = 0.01
FRAMES_TO_READ = int(SAMPLE_RATE * READ_INTERVAL)
SECONDS_TO_READ = 10.0

for _ in range (int(SECONDS_TO_READ / READ_INTERVAL)):
  buffer = speaker.read_frames(FRAMES_TO_READ)
  time.sleep(READ_INTERVAL)
```

   can be replaced with:

```python
SECONDS_TO_READ = 10
FRAMES_TO_READ = SAMPLE_RATE * SECONDS_TO_READ
buffer = speaker.read_frames(FRAMES_TO_READ)
```

### Fixed

- Fixed an issue that was causing sporadic audio gaps on macOS and in certain OS
  task scheduling scenarios.

- Network re-connections have been improved.

## [0.1.1] - 2023-09-27

### Fixed

- Fixed an issue where virtual devices could cause other Python threads to be
  blocked.
