#!/usr/bin/env python3
"""Send enough `set` commands to push the active data file past MAX_FILE_SIZE (1 MiB),
then `get` every key back and check it matches what was set."""
import socket
import sys

HOST, PORT = "0.0.0.0", 8000
TARGET_BYTES = int(sys.argv[1]) if len(sys.argv) > 1 else 1024 * 1024 + 100_000  # a bit over 1 MiB
VALUE_SIZE = 1000
HEADER = 24  # timestamp | key_len | val_len


def make_value(i):
    # unique per key so a read from the wrong offset/file is caught
    prefix = f"v{i}-"
    return prefix + "x" * (VALUE_SIZE - len(prefix))


expected = {}
written = 0
i = 0

with socket.create_connection((HOST, PORT)) as sock:
    f = sock.makefile("rw", newline="\n")
    while written < TARGET_BYTES:
        key = f"key{i}"
        value = make_value(i)
        f.write(f"set {key} {value}\n")
        f.flush()
        resp = f.readline().strip()
        if resp != "OK":
            print(f"set {key} -> {resp!r}")
            break
        expected[key] = value
        written += HEADER + len(key) + VALUE_SIZE
        i += 1

    print(f"sent {i} sets, ~{written} bytes of records")

    failures = 0
    for key, value in expected.items():
        f.write(f"get {key}\n")
        f.flush()
        resp = f.readline().rstrip("\n")
        if resp != value:
            failures += 1
            if failures <= 10:
                print(f"get {key} -> {resp[:60]!r}, expected {value[:60]!r}")

    f.write("get no_such_key\n")
    f.flush()
    missing = f.readline().strip()
    if not missing.startswith("ERR"):
        failures += 1
        print(f"get no_such_key -> {missing!r}, expected ERR")

print(f"checked {len(expected)} gets: {len(expected) - failures} ok, {failures} failed")
sys.exit(1 if failures else 0)
