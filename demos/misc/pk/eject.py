from daily import *
import sys
import time

URL = "https://paulk.staging.daily.co/test"
OWNER_TOKEN = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJvIjp0cnVlLCJkIjoiNTQwMTM3OGUtMGI1Zi00N2VjLTg5NTMtMzAzNjMyODE3OTJkIiwiaWF0IjoxNjk4OTM3MzEyfQ.MBMOEupfWd3U_rrZDmQ2JwbXARtQPYYU-67pUUBHEig"
ADMIN_TOKEN = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJwIjp7ImNhIjoicCJ9LCJkIjoiNTQwMTM3OGUtMGI1Zi00N2VjLTg5NTMtMzAzNjMyODE3OTJkIiwiaWF0IjoxNjk4OTU1NjM3fQ.AS-8Lq7cKByieh4qZeAJ5fLmr7Fj4cFXgPobedDEwy4"

print("starting script!")
print(Daily)

Daily.init()

call = CallClient()

def on_left(_left_data, _error):
  print("left!")
  sys.exit()

call.set_user_name("ejector")

participant_type = input(
  """
select what kind of participant to join as:
1 - owner
2 - non-owner participant admin
3 - "plain"
  """)

if participant_type == "1":
  token = OWNER_TOKEN
elif participant_type == "2":
  token = ADMIN_TOKEN
else:
  token = None

call.join(URL, token)

# Give time for join to finish and participant presence to come in
time.sleep(3)

input("press any key to start test...")

def on_ejected(_, error):
  if (error):
    print("error ejecting: " + error)
  else:
    print("successfully finished ejecting")

ids = list(call.participants().keys())
ids.remove('local')
print("ejecting participants...", ids)
call.eject_remote_participants(ids, completion=on_ejected)

# Give time for ejection messages to be sent
time.sleep(3)

input("press any key to leave...")

call.leave()

# Give time for leave
time.sleep(1)

input("done.")
