#!/usr/bin/env python3
"""Check that the server handles several clients at once and stays consistent.

  1. idle client:   one client connects and sends nothing; a second client must
                    still get answers (fails on a one-connection-at-a-time server).
  2. parallel sets: N clients write disjoint keys at the same time, enough to
                    push past MAX_FILE_SIZE so rotation happens under contention,
                    then every key is read back from fresh connections.
  3. hot key:       one client keeps overwriting a single key with v0, v1, v2, ...
                    while readers poll it; a reader must never see the version
                    go backwards or a value that was never written.

Every socket has a timeout so a blocked server fails the test instead of hanging it."""
import socket
import sys
import threading
import time

HOST, PORT = "0.0.0.0", 8000
TIMEOUT = 5.0  # seconds per request
CLIENTS = 8
VALUE_SIZE = 1000
HEADER = 24  # timestamp | key_len | val_len
TARGET_BYTES = 1024 * 1024 + 200_000  # a bit over 1 MiB in total across all clients
HOT_WRITES = 300
HOT_READERS = 4


class Client:
    def __init__(self):
        self.sock = socket.create_connection((HOST, PORT), timeout=TIMEOUT)
        self.f = self.sock.makefile("rw", newline="\n")

    def cmd(self, line):
        self.f.write(line + "\n")
        self.f.flush()
        return self.f.readline().rstrip("\n")

    def close(self):
        self.f.close()
        self.sock.close()


def make_value(tag):
    # unique per key so a read from the wrong offset/file is caught
    prefix = f"{tag}-"
    return prefix + "x" * (VALUE_SIZE - len(prefix))


def run_threads(target, n):
    """Start n threads at the same moment, wait for all, return their errors."""
    barrier = threading.Barrier(n)
    errors = [[] for _ in range(n)]

    def wrapper(idx):
        try:
            barrier.wait()
            target(idx, errors[idx])
        except Exception as e:  # timeouts, resets, ...
            errors[idx].append(f"thread {idx}: {type(e).__name__}: {e}")

    threads = [threading.Thread(target=wrapper, args=(i,)) for i in range(n)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    return [e for errs in errors for e in errs]


def test_idle_client():
    idle = Client()  # connected, never sends anything
    try:
        c = Client()
        start = time.monotonic()
        resp = c.cmd("set idle_probe 1")
        c.close()
    except socket.timeout:
        return ["second client got no reply while another client was idle"]
    finally:
        idle.close()
    if resp != "OK":
        return [f"set idle_probe -> {resp!r}"]
    print(f"    second client answered in {time.monotonic() - start:.3f}s")
    return []


def test_parallel_sets():
    per_client = TARGET_BYTES // CLIENTS // (HEADER + VALUE_SIZE + 8) + 1
    expected = {}
    lock = threading.Lock()

    def writer(idx, errors):
        c = Client()
        for i in range(per_client):
            key = f"c{idx}_k{i}"
            value = make_value(key)
            resp = c.cmd(f"set {key} {value}")
            if resp != "OK":
                errors.append(f"set {key} -> {resp!r}")
                break
            with lock:
                expected[key] = value
        c.close()

    start = time.monotonic()
    errors = run_threads(writer, CLIENTS)
    print(f"    {len(expected)} sets from {CLIENTS} clients in {time.monotonic() - start:.2f}s")

    keys = list(expected)

    def reader(idx, errors):
        c = Client()
        for key in keys[idx::CLIENTS]:
            resp = c.cmd(f"get {key}")
            if resp != expected[key]:
                errors.append(f"get {key} -> {resp[:40]!r}, expected {expected[key][:40]!r}")
        c.close()

    errors += run_threads(reader, CLIENTS)
    return errors


def test_hot_key():
    done = threading.Event()

    def worker(idx, errors):
        c = Client()
        if idx == 0:  # the single writer
            try:
                for v in range(HOT_WRITES):
                    resp = c.cmd(f"set hot {make_value(f'v{v}')}")
                    if resp != "OK":
                        errors.append(f"set hot v{v} -> {resp!r}")
                        break
            finally:
                done.set()  # let readers stop even if the writer dies
        else:
            last = -1
            reads = 0
            while not done.is_set():
                resp = c.cmd("get hot")
                if resp.startswith("ERR"):
                    continue  # writer hasn't written v0 yet
                try:
                    tag = resp.split("-", 1)[0]
                    v = int(tag[1:])
                    ok = resp == make_value(tag)
                except ValueError:
                    ok = False
                if not ok:
                    errors.append(f"reader {idx}: garbled value {resp[:40]!r}")
                    break
                if v < last:
                    errors.append(f"reader {idx}: saw v{v} after v{last}")
                    break
                last = v
                reads += 1
            print(f"    reader {idx}: {reads} reads, last saw v{last}")
        c.close()

    return run_threads(worker, 1 + HOT_READERS)


TESTS = [
    ("idle client does not block others", test_idle_client),
    ("parallel sets across file rotation", test_parallel_sets),
    ("hot key never goes backwards", test_hot_key),
]

failed = 0
for name, fn in TESTS:
    print(f"[{name}]")
    try:
        errors = fn()
    except Exception as e:
        errors = [f"{type(e).__name__}: {e}"]
    for e in errors[:10]:
        print(f"    {e}")
    if len(errors) > 10:
        print(f"    ... and {len(errors) - 10} more")
    print("    PASS" if not errors else f"    FAIL ({len(errors)} errors)")
    failed += bool(errors)

print(f"{len(TESTS) - failed}/{len(TESTS)} passed")
sys.exit(1 if failed else 0)
