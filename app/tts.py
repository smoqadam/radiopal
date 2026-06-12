from __future__ import annotations

import pathlib

from google.cloud import texttospeech

HERE = pathlib.Path(__file__).resolve().parent
GENERATED_DIR = HERE / "media" / "generated"

DEFAULT_VOICE = "en-US-Chirp3-HD-Charon"
DEFAULT_LANG = "en-US"


def list_voices(language_code):
    client = texttospeech.TextToSpeechClient()
    voices = client.list_voices(language_code=language_code).voices
    chirp = [v for v in voices if "Chirp3-HD" in v.name]
    if not chirp:
        print(f"No Chirp 3 HD voices found for '{language_code}'. "
              f"({len(voices)} other voices exist for this language.)")
        return
    print(f"Chirp 3 HD voices for '{language_code}':")
    for v in sorted(chirp, key=lambda v: v.name):
        print(f"  {v.name}  ({texttospeech.SsmlVoiceGender(v.ssml_gender).name})")


def synthesize(text, voice, language_code, out_path):
    client = texttospeech.TextToSpeechClient()
    response = client.synthesize_speech(
        input=texttospeech.SynthesisInput(text=text),
        voice=texttospeech.VoiceSelectionParams(language_code=language_code, name=voice),
        audio_config=texttospeech.AudioConfig(audio_encoding=texttospeech.AudioEncoding.MP3),
    )
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(response.audio_content)
    return out_path
