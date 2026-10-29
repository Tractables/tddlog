#!/usr/bin/env python3
"""memcap.py: a command's resident memory, capped and measured in a cgroup.

A run caps every system at the same resident memory: each command runs in a
cgroup v2 of its own, whose `memory.max` is the cap, with no swap and
`memory.oom.group` set, so that a process tree that reaches the cap is
killed whole by the kernel and the run reads the kill in `memory.events`.
An address-space limit (`ulimit -v`) counts reservations rather than
memory: a JVM's heap, a Julia process's pools and the new block of a
`realloc` all count against it before any of it is used, so it refused
allocations far below the memory the processes held.

The cgroups are made below one delegated to the user. The run starts in a
user scope with `Delegate=yes`:

    memcap.py delegate -- CMD...

which is `systemd-run --user --scope -p Delegate=yes CMD...`. `Root()` then
moves the scope's processes into a leaf `driver` and hands the memory
controller to the scope's children (cgroup v2 keeps processes out of a
cgroup that hands controllers to its children), and `Cap` makes a child
`run-PID-N` for one command.

A command enters its cgroup before it starts: `Cap.command` runs it through
a shell that writes its own pid into the cgroup and then execs it, so that
everything it allocates and every process it starts is counted.

The peak is the cgroup's `memory.peak` where the kernel keeps one (Linux 6.0
and later), else the largest `memory.current` read every 0.2 s while the
command runs, which can miss a peak shorter than that. Both count what the
cap counts: the tree's anonymous memory, the page cache its reads brought
in (which the kernel reclaims before it kills) and its kernel memory."""
import errno
import itertools
import os
import shutil
import sys
import threading
import time

CGROUP = '/sys/fs/cgroup'
SAMPLE_S = 0.2


class MemcapError(Exception):
    pass


def read(path):
    with open(path) as f:
        return f.read()


def write(path, text):
    with open(path, 'w') as f:
        f.write(text)


def size(text):
    """Bytes of a size written as Java's options are, `31g`, `512m`, `64k`, or
    a number of bytes."""
    units = {'k': 1 << 10, 'm': 1 << 20, 'g': 1 << 30, 't': 1 << 40}
    t = text.strip().lower()
    try:
        return int(t[:-1]) * units[t[-1]] if t and t[-1] in units else int(t)
    except ValueError:
        raise ValueError(f'not a size: {text!r} (a number of bytes, or one with k, m, g or t)') from None


class Root:
    """The delegated cgroup of the run: its processes in the leaf `driver`,
    the memory controller handed to its children. Several drivers of one
    scope (a runner's lanes, each a bench.py) share it."""

    def __init__(self):
        rel = next((line[3:].strip() for line in read('/proc/self/cgroup').splitlines()
                    if line.startswith('0::')), None)
        if rel is None:
            raise MemcapError('no cgroup v2 hierarchy (/proc/self/cgroup has no 0:: line)')
        base = CGROUP + rel
        if os.path.basename(base) == 'driver':   # an earlier Root of the same scope moved this process
            base = os.path.dirname(base)
        control = os.path.join(base, 'cgroup.subtree_control')
        if not (os.access(control, os.W_OK) and os.access(os.path.join(base, 'cgroup.procs'), os.W_OK)):
            raise MemcapError(f'the cgroup {rel} is not delegated to this user: start the run with '
                              '`memcap.py delegate -- CMD...`')
        if 'memory' not in read(os.path.join(base, 'cgroup.controllers')).split():
            raise MemcapError(f'the cgroup {rel} has no memory controller to hand to its children')
        driver = os.path.join(base, 'driver')
        os.makedirs(driver, exist_ok=True)
        # A process that the scope starts while the others move lands in the
        # scope itself and keeps the controller from being handed down
        # (EBUSY): move again and retry.
        for _ in range(100):
            for pid in read(os.path.join(base, 'cgroup.procs')).split():
                try:
                    write(os.path.join(driver, 'cgroup.procs'), pid)
                except ProcessLookupError:
                    pass   # it exited
            if 'memory' in read(control).split():
                break
            try:
                write(control, '+memory')
                break
            except OSError as e:
                if e.errno != errno.EBUSY:
                    raise
                time.sleep(0.01)
        else:
            raise MemcapError(f'the memory controller of {rel} could not be handed to its children')
        self.base = base


class Cap:
    """One command's cgroup: `memory.max` the cap, no swap, the whole tree
    killed at the cap. `command` wraps the command so that it enters the
    cgroup before it starts, `start` follows its memory once it runs, and
    `finish`, once its process has exited, returns its peak and whether the
    cap killed it, kills what it left and removes the cgroup.

    With `group=False` the kernel kills only the process it picks at the cap
    (the one with the most memory), and the others go on: a driver that runs
    a system and writes its record (an earlier commit's bench.py under the
    cap, or a wrapper that runs one) then records the system's failure."""
    _ids = itertools.count()

    def __init__(self, root, limit, group=True):
        self.limit = limit
        self.path = os.path.join(root.base, f'run-{os.getpid()}-{next(Cap._ids)}')
        os.mkdir(self.path)
        try:
            write(self.file('memory.max'), str(limit))
            if os.path.exists(self.file('memory.swap.max')):
                write(self.file('memory.swap.max'), '0')
            write(self.file('memory.oom.group'), '1' if group else '0')
        except OSError:
            os.rmdir(self.path)
            raise
        self.peak = self.file('memory.peak') if os.path.exists(self.file('memory.peak')) else None
        self.sampled = 0
        self.stop = threading.Event()
        self.sampler = None

    def file(self, name):
        return os.path.join(self.path, name)

    def command(self, cmd):
        return ['/bin/sh', '-c', 'echo $$ > "$0/cgroup.procs" && exec "$@"', self.path, *cmd]

    def start(self):
        if self.peak is None:
            self.sampler = threading.Thread(target=self.sample, daemon=True)
            self.sampler.start()

    def sample(self):
        current = self.file('memory.current')
        while True:
            try:
                self.sampled = max(self.sampled, int(read(current)))
            except (OSError, ValueError):
                return
            if self.stop.wait(SAMPLE_S):
                return

    def finish(self):
        """(peak bytes, killed by the cap)."""
        self.stop.set()
        if self.sampler is not None:
            self.sampler.join()
        try:
            events = dict(line.split() for line in read(self.file('memory.events')).splitlines())
            killed = int(events.get('oom_kill', 0)) > 0 or int(events.get('oom_group_kill', 0)) > 0
            peak = int(read(self.peak)) if self.peak else max(self.sampled, int(read(self.file('memory.current'))))
        finally:
            self.remove()
        return peak, killed

    def remove(self):
        """Kills what the command left in the cgroup (a process that left
        its session escapes the run's kill of the group) and removes it."""
        if os.path.exists(self.file('cgroup.kill')):
            write(self.file('cgroup.kill'), '1')
        else:
            for pid in read(self.file('cgroup.procs')).split():
                try:
                    os.kill(int(pid), 9)
                except ProcessLookupError:
                    pass
        for _ in range(500):
            try:
                os.rmdir(self.path)
                return
            except OSError as e:
                if e.errno != errno.EBUSY:
                    raise
                time.sleep(0.01)
        raise MemcapError(f'{self.path}: its processes did not exit within 5 s of the kill')


def delegate(cmd):
    """Execs cmd in a new user scope that delegates its cgroup to the user."""
    run = shutil.which('systemd-run')
    if run is None:
        raise MemcapError('no systemd-run on the PATH')
    os.environ.setdefault('XDG_RUNTIME_DIR', f'/run/user/{os.getuid()}')
    os.execv(run, [run, '--user', '--scope', '--quiet', '--collect', '-p', 'Delegate=yes', '--', *cmd])


def main(argv):
    if len(argv) < 3 or argv[0] != 'delegate' or argv[1] != '--':
        sys.exit('usage: memcap.py delegate -- CMD...')
    try:
        delegate(argv[2:])
    except MemcapError as e:
        sys.exit(f'memcap.py: {e}')


if __name__ == '__main__':
    main(sys.argv[1:])
