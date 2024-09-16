# Changelog

All notable changes to the **daily-python** SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Added missing dialin/dialout handlers in `daily.pyi`.

- `EventHandler.on_dialin_answered` should be
  `EventHandler.on_dialout_answered`.

- Fixed a crash caused by passing non-serializable data to
  `CallClient.send_app_message`.

- Fixed `daily.pyi` type completions.

## [0.10.1] - 2024-06-24

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
