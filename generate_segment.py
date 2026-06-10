#!/usr/bin/env python3
"""RadioPal segment generator.

Pipeline: text -> Chirp 3 HD TTS -> mp3 in ./generated -> (optional) push into
liquidsoap's interrupt queue so it cuts into the music immediately.

Auth: uses Application Default Credentials. Point GOOGLE_APPLICATION_CREDENTIALS
at a service-account JSON, or run `gcloud auth application-default login`.

Examples:
  # discover which Chirp 3 HD voices exist for a language (run this first)
  python generate_segment.py --list-voices --lang fa-IR

  # render a clip and push it on air now
  python generate_segment.py --text "Good morning, you're listening to RadioPal." --push

  # render only, no push (inspect the file in ./generated first)
  python generate_segment.py --name weather --text "Tehran is sunny, 28 degrees." --no-push
"""
from __future__ import annotations

import argparse
import pathlib
import socket
import sys

from google.auth.exceptions import DefaultCredentialsError
from google.cloud import texttospeech

HERE = pathlib.Path(__file__).resolve().parent
GENERATED_DIR = HERE / "generated"          # host path (bind-mounted to /generated in the container)
CONTAINER_DIR = "/generated"                # how liquidsoap sees the same files
TELNET_HOST, TELNET_PORT = "localhost", 1234

# A reasonable default. Swap via --voice; use --list-voices to see what's available
# for your language (Chirp 3 HD voices are named like "<lang>-Chirp3-HD-<Name>").
DEFAULT_VOICE = "en-US-Chirp3-HD-Charon"
DEFAULT_LANG = "en-US"


def list_voices(language_code: str) -> None:
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


def synthesize(text: str, voice: str, language_code: str, out_path: pathlib.Path) -> pathlib.Path:
    client = texttospeech.TextToSpeechClient()
    response = client.synthesize_speech(
        input=texttospeech.SynthesisInput(text=text),
        voice=texttospeech.VoiceSelectionParams(language_code=language_code, name=voice),
        # Chirp 3 HD is plain-text only (no SSML) and supports MP3 output.
        audio_config=texttospeech.AudioConfig(audio_encoding=texttospeech.AudioEncoding.MP3),
    )
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(response.audio_content)
    return out_path


def push_interrupt(container_path: str) -> str:
    """Tell liquidsoap (via its telnet server) to cut this clip into the stream now."""
    payload = f"interrupt.push {container_path}\nquit\n".encode()
    with socket.create_connection((TELNET_HOST, TELNET_PORT), timeout=3) as sock:
        sock.sendall(payload)
        chunks = []
        while True:
            data = sock.recv(4096)
            if not data:
                break
            chunks.append(data)
    return b"".join(chunks).decode(errors="replace").strip()


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description="RadioPal: text -> Chirp 3 HD mp3 -> interrupt queue")
    p.add_argument("--text", help="Text to speak")
    p.add_argument("--name", default="segment", help="Output basename (-> generated/<name>.mp3)")
    p.add_argument("--voice", default=DEFAULT_VOICE, help="Chirp 3 HD voice name")
    p.add_argument("--lang", default=DEFAULT_LANG, help="BCP-47 language code (e.g. en-US, fa-IR)")
    p.add_argument("--list-voices", action="store_true", help="List Chirp 3 HD voices for --lang and exit")
    push = p.add_mutually_exclusive_group()
    push.add_argument("--push", dest="push", action="store_true", help="Push onto the stream now (default)")
    push.add_argument("--no-push", dest="push", action="store_false", help="Render only, don't push")
    p.set_defaults(push=True)
    args = p.parse_args(argv)

    try:
        if args.list_voices:
            list_voices(args.lang)
            return 0

        if not args.text:
            p.error("--text is required (unless using --list-voices)")

        out = GENERATED_DIR / f"{args.name}.mp3"
        synthesize(args.text, args.voice, args.lang, out)
        print(f"Wrote {out} ({out.stat().st_size} bytes)")

        if args.push:
            reply = push_interrupt(f"{CONTAINER_DIR}/{args.name}.mp3")
            print(f"Pushed to interrupt queue. liquidsoap replied: {reply!r}")
        else:
            print("Skipped push (--no-push).")
        return 0
    except DefaultCredentialsError:
        print(
            "No Google Cloud credentials found.\n"
            "Set up one of:\n"
            "  - export GOOGLE_APPLICATION_CREDENTIALS=$PWD/creds/<your-key>.json\n"
            "  - gcloud auth application-default login\n"
            "See https://cloud.google.com/docs/authentication/external/set-up-adc",
            file=sys.stderr,
        )
        return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
