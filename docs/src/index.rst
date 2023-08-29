Daily Client SDK for Python
=======================================

Welcome to `Daily`_'s Python SDK documentation!

The Daily Client SDK for Python allows you to build video and audio calling into
your native desktop and server applications.

This SDK is specially suited to build AI applications on the server side as it
can be easily integrated with well-known Python libraries like `numpy`_ and
`PyTorch`_.

These are some of the main features:

* Join a `Daily`_ meeting.
* Receive video frames from remote participants in your desired color format.
* Simulate local cameras and send video content as a participant.
* Send and receive audio to and from the meeting.

This could be applied to several AI uses cases, for example:

* Perform object or face detection on the server side.
* Generative AI for video content.
* Process audio with a Speech-To-Text platform.
* Send audio coming from a Text-To-Speech platform.


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
