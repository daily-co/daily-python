[![PyPI](https://img.shields.io/pypi/v/daily-python)](https://pypi.org/project/daily-python)
[![Docs](https://img.shields.io/badge/API-docs-00CC00)](https://reference-python.daily.co/)

# Daily Client SDK for Python

> **Looking to develop voice and video agents?**
>
> Check out our voice and multimodal conversational AI framework [Pipecat](https://github.com/pipecat-ai/pipecat), which has excellent support for Daily and already uses this Python SDK.

The Daily client SDK for Python allows you to build video and audio calling into your native desktop and server applications.

![daily-python](https://github.com/daily-co/daily-python/blob/main/daily-python.gif?raw=true)

This SDK is well suited to build AI applications on the server side as it can be
easily integrated with well-known Python libraries such as
[OpenAI](https://github.com/openai/openai-python),
[Deepgram](https://github.com/deepgram/deepgram-python-sdk),
[YOLOv5](https://github.com/ultralytics/yolov5), [PyTorch](https://pytorch.org),
[OpenCV](https://opencv.org/) and much more.

The SDK's core features include:

- Joining a Daily meeting as a participant
- As a meeting participant, configuring inputs, publishing, and subscription settings
- Receiving video frames from other participants
- Receiving raw audio from all participants in the meeting
- Sending video into a meeting
- Sending raw audio into a meeting

This functionality can be applied to several AI use cases, including:

- Performing object or face detection on the server side
- Processing audio from a Speech-To-Text platform into a meeting
- Sending audio from a Text-To-Speech platform into a meeting
- Sending video and audio tracks to a content moderation platform
- Using generative AI to inject video content into a meeting

## Documentation

See the [Daily Python API docs](https://reference-python.daily.co/index.html).

For demos on how to use `daily-python`, refer to the [demos](https://github.com/daily-co/daily-python/tree/main/demos) directory.

## Installation

`daily-python` can be easily installed using `pip`:

```bash
   pip install daily-python
```

To upgrade:

```bash
   pip install -U daily-python
```

### Requirements

- Python 3.7 or newer

## Usage

For usage details, visit Daily's [Python SDK getting start guide](https://docs.daily.co/guides/products/ai-toolkit).

## Support

Need help or have feedback? You can reach out through our [developer community](https://community.daily.co/) or [chat](https://www.daily.co/company/contact/support/) with our support engineers.
