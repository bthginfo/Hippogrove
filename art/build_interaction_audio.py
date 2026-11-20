"""Generate the original Hippogrove handling and creature sounds."""
from math import exp, sin, tau
from pathlib import Path
from random import Random
import struct
import wave

RATE = 24000
OUTPUT = Path(__file__).resolve().parents[1] / "assets/audio"
noise = Random(274)


def write(name, seconds, sample):
    values = []
    for frame in range(round(seconds * RATE)):
        t = frame / RATE
        edge = min(1, t / 0.008, (seconds - t) / 0.022)
        values.append(round(max(-1, min(1, sample(t) * edge)) * 32767))
    with wave.open(str(OUTPUT / f"{name}.wav"), "wb") as stream:
        stream.setparams((1, 2, RATE, 0, "NONE", "not compressed"))
        stream.writeframes(struct.pack(f"<{len(values)}h", *values))


def lift(t):
    breath = noise.uniform(-1, 1) * (exp(-t * 30) - exp(-t * 190))
    jump = sin(tau * (174 * t + 480 * t * t)) * exp(-t * 13)
    return 0.13 * breath + 0.42 * jump


def land(t):
    floor = sin(tau * (106 * t - 95 * t * t)) * exp(-t * 19)
    bundles = sin(tau * 278 * t) * exp(-t * 49)
    brush = noise.uniform(-1, 1) * exp(-t * 38)
    return 0.53 * floor + 0.12 * bundles + 0.14 * brush


def snuffle(t):
    swell = (1 - exp(-t * 33)) * exp(-t * 4.7)
    phase = tau * (88 * t - 22 * t * t + 0.24 * sin(tau * 3.7 * t))
    body = 0.38 * sin(phase) + 0.13 * sin(2 * phase) + 0.08 * sin(3 * phase)
    fur = noise.uniform(-1, 1) * (0.72 + 0.28 * sin(tau * 4 * t))
    return swell * (body + 0.10 * fur)


def chime(t):
    note = 0.
    for index, frequency in enumerate((392., 493.88, 587.33)):
        age = t - 0.12 * index
        if age >= 0:
            decay = min(age / 0.018, 1) * exp(-age * 5.8)
            note += decay * (0.24 * sin(tau * frequency * age) + 0.07 * sin(tau * 2 * frequency * age))
    return note


def omen(t):
    pulse = min(t / 0.03, 1) * exp(-t * 4.2)
    age = max(t - 0.16, 0)
    second = min(age / 0.04, 1) * exp(-age * 5.5)
    return 0.36 * pulse * sin(tau * 164.81 * t) + 0.23 * second * sin(tau * 123.47 * age)


if __name__ == "__main__":
    for clip in [
        ("paw_lift", 0.28, lift),
        ("paw_land", 0.32, land),
        ("hippo_care", 0.78, snuffle),
        ("chapter_chime", 0.88, chime),
        ("rival_omen", 0.81, omen),
    ]:
        write(*clip)
