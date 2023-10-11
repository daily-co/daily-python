# daily-python Flask/Celery demo

This is a Flask application that allows you to connect bots to a Daily meeting
by making POST requests to a URL. The bots will synthesize sentences to audio
into the meeting.

# Dependencies

The application needs Flask, Celery and Redis Python packages installed. It also
assumes you have a local running Redis server.

To install the Python packages you can type:

```bash
pip3 install flask celery redis
```

Installing a Redis server might be specific to your operating system.

# Usage

Once all the dependencies are installed we will first run the Celery worker in
one terminal:

```bash
celery -A app.celery worker --loglevel INFO
```

Then we will run the Flask application:

```bash
flask run
```

# Making requests

The body of the request is a JSON object with the following fields:

```json
{
  "bot_name": "BOT_NAME",
  "meeting_url": "DAILY_MEETING_URL"
}
```

We can easily make a request with `curl`:

```bash
curl -d '{"bot_name": "BOT_NAME", "meeting_url":"DAILY_MEETING_URL"}' -H "Content-Type: application/json" -X POST http://localhost:5000
```

This will be received by the Flask application and a new process will be
created.
