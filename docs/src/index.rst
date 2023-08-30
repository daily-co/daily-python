Daily Client SDK for Python
=======================================

Welcome to `Daily`_'s Python client SDK documentation!

The Daily Client SDK for Python allows you to build video and audio calling into
your native desktop and server applications.

This SDK is well suited to build AI applications on the server side as it can be
easily integrated with well-known Python libraries like `numpy`_ and `PyTorch`_.

The core features of the SDK include:

* Join a `Daily`_ meeting as a participant with the ability to receive media
  from and send media to other participants.
* Simulate local cameras and send video content.
* Send and receive audio to and from the meeting.

This could be applied to several AI uses cases, for example:

* Perform object or face detection on the server side.
* Generative AI for video content.
* Process audio with a Speech-To-Text platform.
* Send audio coming from a Text-To-Speech platform.
* Send video to third-party services for processing.

.. note::

   This SDK is in pre-beta. This means that the API might change before we
   the official release.

.. _Daily: https://daily.co
.. _numpy: https://numpy.org
.. _PyTorch: https://pytorch.org

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
