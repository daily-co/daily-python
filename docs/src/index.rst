Daily Client SDK for Python
=======================================

Welcome to `Daily`_'s Python client SDK documentation!

The Daily Client SDK for Python allows you to build video and audio calling into
your native desktop and server applications.

This SDK is well suited to build AI applications on the server side as it can be
easily integrated with well-known Python libraries such as `YOLOv5`_,
`PyTorch`_, `OpenCV`_ or `numpy`_.

The core features of the SDK include:

* Join a Daily meeting as a participant
* As a meeting participant, configure inputs, publishing, and subscription settings
* Receive video frames from other participants
* Receive raw audio from all participants in the meeting
* Send raw audio into a meeting

This functionality can be applied to several AI use cases, including:

* Perform object or face detection on the server side
* Process audio from a Speech-To-Text platform into a meeting
* Send audio from a Text-To-Speech platform into a meeting
* Send video and audio tracks to a content moderation platform
* Use generative AI to inject video content into a meeting

.. note::

   This SDK is in pre-beta. This means that the API might change before we
   the official release.

.. _Daily: https://daily.co
.. _numpy: https://numpy.org
.. _OpenCV: https://opencv.org/
.. _PyTorch: https://pytorch.org
.. _YOLOv5: https://github.com/ultralytics/yolov5

.. toctree::
   :maxdepth: 2

   installation
   usage
   api_reference


Indices and tables
=======================================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`
