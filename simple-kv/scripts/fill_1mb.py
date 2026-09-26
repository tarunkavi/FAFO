#!/usr/bin/env python3
"""Send enough `set` commands to push the active data file past MAX_FILE_SIZE (1 MiB)."""
import socket
import sys

HOST, PORT = "0.0.0.0", 8000
TARGET_BYTES = int(sys.argv[1]) if len(sys.argv) > 1 else 1024 * 1024 + 100_000  # a bit over 1 MiB
VALUE_SIZE = 1000
HEADER = 24  # timestamp | key_len | val_len

value = "v" * VALUE_SIZE
written = 0
i = 0

with socket.create_connection((HOST, PORT)) as sock:
    f = sock.makefile("rw", newline="\n")
    while written < TARGET_BYTES:
        key = f"key{i}"
        f.write(f"set {key} {value}\n")
        f.flush()
        resp = f.readline().strip()
        if resp != "OK":
            print(f"set {key} -> {resp!r}")
            break
        written += HEADER + len(key) + VALUE_SIZE
        i += 1

print(f"sent {i} sets, ~{written} bytes of records")
