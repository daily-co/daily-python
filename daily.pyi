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
    def create_camera_device(
            device_name: str,
            width: int,
            height: int,
            color_format: str = "RGBA") -> VirtualCameraDevice:
        ...

    @staticmethod
    def create_speaker_device(
            device_name: str,
            sample_rate: int = 16000,
            channels: int = 1,
            non_blocking: bool = False) -> VirtualSpeakerDevice:
        ...

    @staticmethod
    def create_microphone_device(
            device_name: str,
            sample_rate: int = 16000,
            channels: int = 1,
            non_blocking: bool = False) -> VirtualMicrophoneDevice:
        ...

    @staticmethod
    def create_native_vad(
            reset_period_ms: int = 500,
            sample_rate: int = 16000,
            channels: int = 1) -> NativeVad:
        ...

    @staticmethod
    def select_speaker_device(device_name: str) -> None:
        ...


class CallClient:

    def __init__(self, event_handler: Optional[EventHandler] = None) -> None:
        ...

    def release(self) -> None:
        ...

    def join(self,
             meeting_url: str,
             meeting_token: Optional[str] = None,
             client_settings: Optional[Mapping[str,
                                               Any]] = None,
             completion: Optional[Callable[[Optional[Mapping[str,
                                                             Any]],
                                            Optional[str]],
                                           None]] = None) -> None:
        ...

    def leave(self, completion: Optional[Callable[[
              Optional[str]], None]] = None) -> None:
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
                                   completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...

    def eject_remote_participants(
            self, ids: Sequence[str], completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...

    def inputs(self) -> Mapping[str, Any]:
        ...

    def update_inputs(self,
                      input_settings: Mapping[str, Any],
                      completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...

    def publishing(self) -> Mapping[str, Any]:
        ...

    def update_publishing(self,
                          publishing_settings: Mapping[str, Any],
                          completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...

    def subscriptions(self) -> Mapping[str, Any]:
        ...

    def update_subscriptions(self,
                             participant_settings: Optional[Mapping[str, Any]] = None,
                             profile_settings: Optional[Mapping[str, Any]] = None,
                             completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...

    def subscription_profiles(self) -> Mapping[str, Any]:
        ...

    def update_subscription_profiles(self,
                                     profile_settings: Mapping[str,
                                                               Any],
                                     completion: Optional[Callable[[Optional[str]],
                                                                   None]] = None) -> None:
        ...

    def update_permissions(self,
                           permissions: Mapping[str, Any],
                           completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...

    def start_recording(self,
                        streaming_settings: Optional[Mapping[str, Any]] = None,
                        stream_id: Optional[str] = None,
                        force_new: Optional[bool] = None,
                        completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...

    def stop_recording(self,
                       stream_id: Optional[str] = None,
                       completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...

    def update_recording(self,
                         update_settings: Optional[Mapping[str, Any]] = None,
                         stream_id: Optional[str] = None,
                         completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...

    def start_transcription(self,
                            settings: Optional[Mapping[str, Any]] = None,
                            completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...

    def stop_transcription(
            self, completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...

    def start_dialout(self,
                      settings: Optional[Mapping[str, Any]] = None,
                      completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...

    def stop_dialout(self, completion: Optional[Callable[[
                     Optional[str]], None]] = None) -> None:
        ...

    def send_app_message(self,
                         message: Any,
                         participant: Optional[str] = None,
                         completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...

    def send_prebuilt_chat_message(self,
                                   message: str,
                                   user_name: Optional[str] = None,
                                   completion: Optional[Callable[[Optional[str]],
                                                                 None]] = None) -> None:
        ...

    def get_network_stats(self) -> Mapping[str, Any]:
        ...

    def set_audio_renderer(self,
                           participant_id: str,
                           callback: Callable[[str, AudioData], None],
                           audio_source: str = "microphone") -> None:
        ...

    def set_video_renderer(self,
                           participant_id: str,
                           callback: Callable[[str, VideoFrame], None],
                           video_source: str = "camera",
                           color_format: str = "RGBA") -> None:
        ...

    def set_proxy_url(self,
                      proxy_url: Optional[str] = None,
                      completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...

    def set_ice_config(self,
                       ice_config: Optional[Mapping[str, Any]] = None,
                       completion: Optional[Callable[[Optional[str]], None]] = None) -> None:
        ...


class EventHandler:

    def __init__(self) -> None:
        ...

    def on_active_speaker_changed(
            self, participant: Mapping[str, Any]) -> None:
        ...

    def on_app_message(self, message: Any, sender: str) -> None:
        ...

    def on_available_devices_updated(
            self, available_devices: Mapping[str, Any]) -> None:
        ...

    def on_call_state_updated(self, state: str) -> None:
        ...

    def on_dialin_ready(self, sip_endpoint: str) -> None:
        ...

    def on_dialout_answered(self, data: Mapping[str, Any]) -> None:
        ...

    def on_dialout_connected(self, data: Mapping[str, Any]) -> None:
        ...

    def on_dialout_error(self, data: Mapping[str, Any]) -> None:
        ...

    def on_dialout_stopped(self, data: Mapping[str, Any]) -> None:
        ...

    def on_dialout_warning(self, data: Mapping[str, Any]) -> None:
        ...

    def on_error(self, message: str) -> None:
        ...

    def on_inputs_updated(self, input_settings: Mapping[str, Any]) -> None:
        ...

    def on_live_stream_error(self, stream_id: str, message: str) -> None:
        ...

    def on_live_stream_started(self, status: Mapping[str, Any]) -> None:
        ...

    def on_live_stream_stopped(self, stream_id: str) -> None:
        ...

    def on_live_stream_warning(self, stream_id: str, message: str) -> None:
        ...

    def on_network_stats_updated(self, stats: Mapping[str, Any]) -> None:
        ...

    def on_participant_counts_updated(self, counts: Mapping[str, Any]) -> None:
        ...

    def on_participant_joined(self, participant: Mapping[str, Any]) -> None:
        ...

    def on_participant_left(self,
                            participant: Mapping[str, Any],
                            reason: str) -> None:
        ...

    def on_participant_updated(self, participant: Mapping[str, Any]) -> None:
        ...

    def on_publishing_updated(
            self, publishing_settings: Mapping[str, Any]) -> None:
        ...

    def on_recording_error(self, stream_id: str, message: str) -> None:
        ...

    def on_recording_started(self, status: Mapping[str, Any]) -> None:
        ...

    def on_recording_stopped(self, stream_id: str) -> None:
        ...

    def on_subscription_profiles_updated(
            self, subscription_profiles: Mapping[str, Any]) -> None:
        ...

    def on_subscriptions_updated(
            self, subscriptions: Mapping[str, Any]) -> None:
        ...

    def on_transcription_error(self, message: str) -> None:
        ...

    def on_transcription_message(self, message: Mapping[str, Any]) -> None:
        ...

    def on_transcription_started(self, status: Mapping[str, Any]) -> None:
        ...

    def on_transcription_stopped(self,
                                 stopped_by: str,
                                 stopped_by_error: bool) -> None:
        ...


class AudioData:

    @ property
    def bits_per_sample(self) -> int:
        ...

    @ property
    def sample_rate(self) -> int:
        ...

    @ property
    def num_channels(self) -> int:
        ...

    @ property
    def num_audio_frames(self) -> int:
        ...

    @ property
    def audio_frames(self) -> bytes:
        ...


class VideoFrame:

    @ property
    def buffer(self) -> bytes:
        ...

    @ property
    def width(self) -> int:
        ...

    @ property
    def height(self) -> int:
        ...

    @ property
    def timestamp_us(self) -> int:
        ...

    @ property
    def color_format(self) -> str:
        ...


class VirtualCameraDevice:

    @ property
    def name(self) -> str:
        ...

    @ property
    def width(self) -> int:
        ...

    @ property
    def height(self) -> int:
        ...

    @ property
    def color_format(self) -> str:
        ...

    def write_frame(self, frame: bytes) -> None:
        ...


class VirtualMicrophoneDevice:

    @ property
    def name(self) -> str:
        ...

    @ property
    def sample_rate(self) -> int:
        ...

    @ property
    def channels(self) -> int:
        ...

    def write_frames(self,
                     frame: bytes,
                     completion: Optional[Callable[[int], None]] = None) -> int:
        ...


class VirtualSpeakerDevice:

    @ property
    def name(self) -> str:
        ...

    @ property
    def sample_rate(self) -> int:
        ...

    @ property
    def channels(self) -> int:
        ...

    def read_frames(self,
                    num_frame: int,
                    completion: Optional[Callable[[bytes], None]] = None) -> bytes:
        ...


class NativeVad:

    @ property
    def rest_period_ms(self) -> int:
        ...

    @ property
    def sample_rate(self) -> int:
        ...

    @ property
    def channels(self) -> int:
        ...

    def analyze_frames(self, frame: bytes) -> float:
        ...
