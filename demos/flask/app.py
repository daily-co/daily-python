from flask import Flask, request

from celery import Celery, Task

from billiard.context import Process

from bot import start_bot

def celery_init_app(app: Flask) -> Celery:
    class FlaskTask(Task):
        def __call__(self, *args: object, **kwargs: object) -> object:
            with app.app_context():
                return self.run(*args, **kwargs)

    celery_app = Celery(app.name, task_cls=FlaskTask)
    celery_app.config_from_object(app.config["CELERY"])
    celery_app.set_default()
    app.extensions["celery"] = celery_app
    return celery_app

app = Flask(__name__)
app.config.from_mapping(
    CELERY=dict(
        broker_url="redis://localhost",
        result_backend="redis://localhost",
        task_ignore_result=True,
    ),
)
app.config.from_prefixed_env()

celery = celery_init_app(app)

@celery.task
def create_bot(bot_name, meeting_url):
    process = Process(target=start_bot, args=(bot_name, meeting_url))
    process.start()
    process.join()

@app.route("/", methods=["POST"])
def new_bot():
    content = request.get_json(silent=True)
    bot_name = content["bot_name"]
    meeting_url = content["meeting_url"]
    create_bot.delay(bot_name, meeting_url)
    return ""
