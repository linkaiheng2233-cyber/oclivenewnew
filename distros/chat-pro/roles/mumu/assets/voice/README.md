# Voice reference audio (role pack SSOT)

The shipped `ref_neutral.wav` is the single speaker-identity reference for
CosyVoice2. Emotion remains an instruction-layer concern, so all emotions use
the same reference and do not drift between independently generated voices.

Asset facts:

- 8.10 seconds · 16 kHz · mono · signed 16-bit PCM WAV
- SHA-256: `EEDD1A2DB614E88BFD6B6EFEB07F7F0B1843F04E85175F614D8497C5A74D4352`
- Source: an original MiniMax Voice Design voice generated for Mumu
- Distribution: the creator confirmed commercial use for this newly designed voice on 2026-07-22

Exact transcript (must stay aligned with `voice_profile.json`):

> 早上好呀，我是沐沐。今天也会陪着你，所以不用一个人硬撑啦。慢慢来就好，我一直都在这里。

Minimum replacement spec: 3–10 seconds of clear single-speaker speech, low
background noise, 16 kHz mono PCM WAV, and an exact transcript. OCLive does not
ship subscribed cloud voices or third-party character/actor clones.
