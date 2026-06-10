from __future__ import annotations

import pathlib
import socket

HOST = "localhost"
PORT = 1234
ROOT = pathlib.Path(__file__).resolve().parent


def command(text):
    payload = f"{text}\nquit\n".encode()
    with socket.create_connection((HOST, PORT), timeout=3) as sock:
        sock.sendall(payload)
        chunks = []
        while True:
            data = sock.recv(4096)
            if not data:
                break
            chunks.append(data)
    return b"".join(chunks).decode(errors="replace").strip()


def push(local_path, lane):
    container = "/" + str(pathlib.Path(local_path).resolve().relative_to(ROOT))
    return command(f"{lane}.push {container}")
