#
# These are type hints for daily-python. They have been MANUALLY generated and
# need to be updated if function signatures are modified in the Rust
# implementation.
#
# See https://docs.python.org/3/library/typing.html
#

from typing import Any, Callable, Mapping, Optional, Sequence

class Daily:

    @staticmethod
    def init(worker_threads: int = 2) -> None:
        ...

    @staticmethod
    def deinit() -> None:
        ...

    @staticmethod
    def create_camera_device(device_name: str,
                             width: int,
                             height: int,
                             color_format: str = "RGBA") -> VirtualCameraDevice:
        ...

    @staticmethod
    def create_speaker_device(device_name: str,
                              sample_rate: int = 16000,
                              channels: int = 1) -> VirtualSpeakerDevice:
        ...

    @staticmethod
    def create_microphone_device(device_name: str,
                                 sample_rate: int = 16000,
                                 channels: int = 1) -> VirtualMicrophoneDevice:
        ...

    @staticmethod
    def select_speaker_device(device_name: str) -> None:
        ...


class CallClient:

    def __init__(self, event_handler: Optional[EventHandler] = None) -> None:
        ...

    def join(self,
             meeting_url: str,
             meeting_token: Optional[str] = None,
             client_settings: Optional[Mapping[str, Any]] = None,
             completion: Optional[Callable[[Optional[Mapping[str, Any]], Optional[str]], None]] = None) -> None:
        ...

    def leave(self, completion: Optional[Callable[[None, Optional[str]], None]] = None) -> None:
        ...

    def set_user_name(self, user_name: str) -> None:
        ...

    def active_speaker(self) -> Mapping[str, Any]:
        ...

    def participants(self) -> Mapping[str, Any]:
        ...

    def participant_counts(self) -> Mapping[str, Any]:
        ...

    def update_remote_participants(self,
                                   remote_participants: Mapping[str, Any],
                                   completion: Optional[Callable[[None, Optional[str]], None]] = None) -> None:
        ...

    def eject_remote_participants(self,
                                  ids: Sequence[str],
                                  completion: Optional[Callable[[None, Optional[str]], None]] = None) -> None:
        ...

    def inputs(self) -> Mapping[str, Any]:
        ...

    def update_inputs(self,
                      input_settings: Mapping[str, Any],
                      completion: Optional[Callable[[Optional[Mapping[str, Any]], Optional[str]], None]] = None) -> None:
        ...

    def publishing(self) -> Mapping[str, Any]:
        ...

    def update_publishing(self,
                          publishing_settings: Mapping[str, Any],
                          completion: Optional[Callable[[Optional[Mapping[str, Any]], Optional[str]], None]] = None) -> None:
        ...

    def subscriptions(self) -> Mapping[str, Any]:
        ...

    def update_subscriptions(self,
                             participant_settings: Optional[Mapping[str, Any]] = None,
                             profile_settings: Optional[Mapping[str, Any]] = None,
                             completion: Optional[Callable[[Optional[Mapping[str, Any]], Optional[str]], None]] = None) -> None:
        ...

    def subscription_profiles(self) -> Mapping[str, Any]:
        ...

    def update_subscription_profiles(self,
                                     profile_settings: Mapping[str, Any],
                                     completion: Optional[Callable[[Optional[Mapping[str, Any]], Optional[str]], None]] = None) -> None:
        ...

    def update_permissions(self,
                           permissions: Mapping[str, Any],
                           completion: Optional[Callable[[None, Optional[str]], None]] = None) -> None:
        ...

    def start_recording(self,
                        streaming_settings: Optional[Mapping[str, Any]] = None,
                        stream_id: Optional[str] = None,
                        force_new: Optional[bool] = None,
                        completion: Optional[Callable[[None, Optional[str]], None]] = None) -> None:
        ...

    def stop_recording(self,
                       stream_id: Optional[str] = None,
                       completion: Optional[Callable[[None, Optional[str]], None]] = None) -> None:
        ...

    def update_recording(self,
                         update_settings: Optional[Mapping[str, Any]] = None,
                         stream_id: Optional[str] = None,
                         completion: Optional[Callable[[None, Optional[str]], None]] = None) -> None:
        ...

    def start_transcription(self,
                            settings: Optional[Mapping[str, Any]] = None,
                            completion: Optional[Callable[[None, Optional[str]], None]] = None) -> None:
        ...

    def stop_transcription(self, completion: Optional[Callable[[None, Optional[str]], None]] = None) -> None:
        ...

    def send_app_message(self,
                         message: Any,
                         participant: Optional[str] = None,
                         completion: Optional[Callable[[None, Optional[str]], None]] = None) -> None:
        ...

    def get_network_stats(self) -> Mapping[str, Any]:
        ...

    def set_video_renderer(self,
                           participant_id: str,
                           callback: Callable[[str, VideoFrame], None],
                           video_source: str = "camera",
                           color_format: str = "RGBA") -> None:
        ...

class EventHandler:

    def __init__(self) -> None:
        ...

    def on_active_speaker_changed(self, participant: Mapping[str, Any]) -> None:
        ...

    def on_app_message(self,
                       message: Mapping[str, Any],
                       sender: Mapping[str, Any]) -> None:
        ...

    def on_available_devices_updated(self,
                                     available_devices: Mapping[str, Any]) -> None:
        ...

    def on_call_state_updated(self, state: Mapping[str, Any]) -> None:
        ...

    def on_error(self, message: Mapping[str, Any]) -> None:
        ...

    def on_inputs_updated(self, input_settings: Mapping[str, Any]) -> None:
        ...

    def on_live_stream_error(self,
                             stream_id: Mapping[str, Any],
                             message: Mapping[str, Any]) -> None:
        ...

    def on_live_stream_started(self, status: Mapping[str, Any]) -> None:
        ...

    def on_live_stream_stopped(self, stream_id: Mapping[str, Any]) -> None:
        ...

    def on_live_stream_warning(self,
                               stream_id: Mapping[str, Any],
                               message: Mapping[str, Any]) -> None:
        ...

    def on_network_stats_updated(self, stats: Mapping[str, Any]) -> None:
        ...

    def on_participant_counts_updated(self, counts: Mapping[str, Any]) -> None:
        ...

    def on_participant_joined(self, participant: Mapping[str, Any]) -> None:
        ...

    def on_participant_left(self,
                            participant: Mapping[str, Any],
                            reason: Mapping[str, Any]) -> None:
        ...

    def on_participant_updated(self, participant: Mapping[str, Any]) -> None:
        ...

    def on_publishing_updated(self, publishing_settings: Mapping[str, Any]) -> None:
        ...

    def on_recording_error(self,
                           stream_id: Mapping[str, Any],
                           message: Mapping[str, Any]) -> None:
        ...

    def on_recording_started(self, status: Mapping[str, Any]) -> None:
        ...

    def on_recording_stopped(self, stream_id: Mapping[str, Any]) -> None:
        ...

    def on_subscription_profiles_updated(self,
                                         subscription_profiles: Mapping[str, Any]) -> None:
        ...

    def on_subscriptions_updated(self, subscriptions: Mapping[str, Any]) -> None:
        ...

    def on_transcription_error(self, message: Mapping[str, Any]) -> None:
        ...

    def on_transcription_message(self, message: Mapping[str, Any]) -> None:
        ...

    def on_transcription_started(self, status: Mapping[str, Any]) -> None:
        ...

    def on_transcription_stopped(self,
                                 stopped_by: Mapping[str, Any],
                                 stopped_by_error: Mapping[str, Any]) -> None:
        ...

class VideoFrame:

    @property
    def buffer(self) -> bytes:
        ...

    @property
    def width(self) -> int:
        ...

    @property
    def height(self) -> int:
        ...

    @property
    def timestamp_us(self) -> int:
        ...

    @property
    def color_format(self) -> str:
        ...

class VirtualCameraDevice:

    @property
    def name(self) -> str:
        ...

    @property
    def width(self) -> int:
        ...

    @property
    def height(self) -> int:
        ...

    @property
    def color_format(self) -> str:
        ...

    def write_frame(self, frame: bytes) -> None:
        ...

class VirtualMicrophoneDevice:

    @property
    def name(self) -> str:
        ...

    @property
    def sample_rate(self) -> int:
        ...

    @property
    def channels(self) -> int:
        ...

    def write_frames(self, frame: bytes) -> None:
        ...

class VirtualSpeakerDevice:

    @property
    def name(self) -> str:
        ...

    @property
    def sample_rate(self) -> int:
        ...

    @property
    def channels(self) -> int:
        ...

    def read_frames(self, num_frame: int) -> bytes:
        ...
