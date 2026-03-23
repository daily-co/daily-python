# wave_audio_receive.py

six 15 second tests:
```bash
for i in {1..6};do timeout 15s uv run python wav_audio_receive.py; done
```
or
```bash
for i in {1..6};do timeout 15s python3 -m wav_audio_receive; done
```