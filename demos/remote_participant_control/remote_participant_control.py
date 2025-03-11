import argparse
import asyncio
import signal

from daily import *


class RemoteParticipantControlApp(EventHandler):
    """An interactive CLI where the user can first:
    - join a Daily call as an owner, or
    - join a Daily call as a regular participant

    Once they're joined, if they're an owner they can then:
    - revoke the first remote participant's canSend permission
    - restore the first remote participant's canSend permission
    - revoke the first remote participant's canReceive permission
    - restore the first remote participant's canReceive permission
    """

    def __init__(self, meeting_url, owner_token):
        self.__meeting_url = meeting_url
        self.__owner_token = owner_token
        self.__client = CallClient(event_handler=self)
        self.__start_event = asyncio.Event()
        self.__task = asyncio.get_running_loop().create_task(self.__cli())

    async def run(self):
        self.__start_event.set()
        await self.__task

    async def stop(self):
        future = asyncio.get_running_loop().create_future()

        def leave_completion(error):
            future.get_loop().call_soon_threadsafe(future.set_result, error)

        self.__client.leave(completion=leave_completion)

        await future

        self.__client.release()

        self.__task.cancel()
        await self.__task

    def on_participant_updated(self, participant):
        info = participant.get("info", {})
        if not info.get("isLocal", True):
            print(
                f"\n\nremote participant updated! permissions: \n{participant.get('info', {}).get('permissions', None)}\n"
            )

    async def __cli(self):
        await self.__start_event.wait()

        try:
            is_owner = await self.__run_cli_join_step()
            if is_owner:
                await self.__run_cli_owner_actions_step()
            else:
                await self.__run_cli_regular_participant_actions_step()
        except asyncio.CancelledError:
            pass

    async def __run_cli_join_step(self) -> bool:
        is_owner = False
        while True:
            print("Choose a join option:")
            print("1. Join as owner")
            print("2. Join as regular participant")
            print("3. Quit")
            join_option = await asyncio.get_event_loop().run_in_executor(
                None, input, "Enter choice: "
            )

            if join_option == "1":
                is_owner = True
                await self.__join(meeting_token=self.__owner_token)
                break
            elif join_option == "2":
                is_owner = False
                await self.__join()
                break
            elif join_option == "3":
                await self.stop()
                break
            else:
                print("Invalid choice")
        return is_owner

    async def __run_cli_owner_actions_step(self):
        while True:
            print("\nChoose an action:")
            print("1. canSend permission: revoke")
            print("2. canSend permission: restore")
            print("3. canReceive permission: revoke")
            print("4. canReceive permission: restore")
            print("5. Quit")
            action = await asyncio.get_event_loop().run_in_executor(None, input, "Enter choice: ")

            if action == "1":
                await self.__revoke_can_send_permission()
            elif action == "2":
                await self.__restore_can_send_permission()
            elif action == "3":
                await self.__revoke_can_receive_permission()
            elif action == "4":
                await self.__restore_can_receive_permission()
            elif action == "5":
                await self.stop()
                break
            else:
                print("Invalid choice")

    async def __run_cli_regular_participant_actions_step(self):
        while True:
            print("\nChoose an action:")
            print("1. Quit")
            action = await asyncio.get_event_loop().run_in_executor(None, input, "Enter choice: ")

            if action == "1":
                await self.stop()
                break
            else:
                print("Invalid choice")

    async def __join(self, meeting_token=None):
        future = asyncio.get_running_loop().create_future()

        def join_completion(data, error):
            future.get_loop().call_soon_threadsafe(future.set_result, (data, error))

        self.__client.join(
            meeting_url=self.__meeting_url,
            meeting_token=meeting_token,
            completion=join_completion,
        )

        return await future

    async def __revoke_can_send_permission(self):
        await self.__update_first_remote_participant({"permissions": {"canSend": []}})

    async def __restore_can_send_permission(self):
        await self.__update_first_remote_participant(
            {
                "permissions": {
                    "canSend": [
                        "camera",
                        "microphone",
                        "screenVideo",
                        "screenAudio",
                        "customVideo",
                        "customAudio",
                    ]
                }
            }
        )

    async def __revoke_can_receive_permission(self):
        await self.__update_first_remote_participant(
            {"permissions": {"canReceive": {"base": False}}}
        )

    async def __restore_can_receive_permission(self):
        await self.__update_first_remote_participant(
            {"permissions": {"canReceive": {"base": True}}}
        )

    async def __update_first_remote_participant(self, updates):
        future = asyncio.get_running_loop().create_future()

        def update_completion(error):
            future.get_loop().call_soon_threadsafe(future.set_result, (error))

        first_participant_id = self.__get_first_remote_participant_id()
        if first_participant_id is None:
            print("No remote participant found; skipping")
        else:
            self.__client.update_remote_participants(
                remote_participants={first_participant_id: updates},
                completion=update_completion,
            )

        return await future

    def __get_first_remote_participant_id(self) -> str | None:
        participants = self.__client.participants()
        return next((key for key in participants.keys() if key != "local"), None)


async def sig_handler(app: RemoteParticipantControlApp):
    print("Ctrl-C detected. Exiting!")
    await app.stop()


async def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--meeting", required=True, help="Meeting URL")
    parser.add_argument("-o", "--owner-token", required=True, help="Owner token")
    args = parser.parse_args()

    Daily.init()

    app = RemoteParticipantControlApp(args.meeting, args.owner_token)

    loop = asyncio.get_running_loop()
    loop.add_signal_handler(signal.SIGINT, lambda *args: asyncio.create_task(sig_handler(app)))

    await app.run()


if __name__ == "__main__":
    asyncio.run(main())
