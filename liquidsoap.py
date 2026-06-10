from __future__ import annotations

import pathlib
import socket

HOST = "localhost"
PORT = 1234
MOUNT = "/generated"


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


def push(local_path):
    name = pathlib.Path(local_path).name
    return command(f"interrupt.push {MOUNT}/{name}")
