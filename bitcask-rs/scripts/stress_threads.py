#!/usr/bin/env python3
"""Open many connections at once and keep them open, so the server ends up with one
live thread per connection. Then check the server is still alive.

  usage: stress_threads.py [N]      (default 5000)

Each connection sends one `get` and waits for the reply, which proves a thread is
actually serving it (a bare connect() can succeed from the kernel backlog alone).

What to look for in the server's terminal when it breaks:
  - "failed to spawn thread"  -> hit the per-process thread limit (sysctl kern.num_taskthreads)
  - "accept failed: Too many open files" -> hit the fd limit (ulimit -n in the server's shell)"""
import resource
import socket
import sys
import time

HOST, PORT = "0.0.0.0", 8000
N = int(sys.argv[1]) if len(sys.argv) > 1 else 5000
TIMEOUT = 5.0

# this script also needs N fds; raise our own soft limit as far as allowed
soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
want = min(hard, N + 100) if hard != resource.RLIM_INFINITY else N + 100
if soft < want:
    resource.setrlimit(resource.RLIMIT_NOFILE, (want, hard))


def probe(label):
    """A fresh client does a set+get; returns True if the server answered."""
    try:
        with socket.create_connection((HOST, PORT), timeout=TIMEOUT) as s:
            f = s.makefile("rw", newline="\n")
            f.write("set stress_probe ok\n"); f.flush()
            a = f.readline().strip()
            f.write("get stress_probe\n"); f.flush()
            b = f.readline().strip()
        ok = (a, b) == ("OK", "ok")
        print(f"[{label}] fresh client: {'answered' if ok else f'bad reply {a!r} {b!r}'}")
        return ok
    except OSError as e:
        print(f"[{label}] fresh client: {type(e).__name__}: {e}")
        return False


probe("before")

held = []
served = 0
failure = None
start = time.monotonic()
for i in range(N):
    try:
        s = socket.create_connection((HOST, PORT), timeout=TIMEOUT)
        held.append(s)
        s.sendall(b"get stress_probe\n")
        if not s.recv(64):
            failure = f"connection {i}: server closed it"
            break
        served += 1
    except OSError as e:
        failure = f"connection {i}: {type(e).__name__}: {e}"
        break
    if (i + 1) % 500 == 0:
        print(f"  {i + 1} connections held, each served by a thread")

print(f"held {served}/{N} live connections in {time.monotonic() - start:.1f}s")
if failure:
    print(f"stopped at {failure}")

alive_loaded = probe(f"with {served} held")

for s in held:
    s.close()
time.sleep(1)  # let the server threads see EOF and exit
alive_after = probe("after release")

ok = served == N and alive_loaded and alive_after
print("PASS" if ok else "FAIL")
sys.exit(0 if ok else 1)
