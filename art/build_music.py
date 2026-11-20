"""Render the original Hippogrove instrumental loop. Requires NumPy, SciPy and FFmpeg."""
from pathlib import Path
import subprocess

import numpy as np
from scipy.signal import butter, sosfilt

RATE = 44100
BEAT = 60 / 76
ROOT = Path(__file__).resolve().parents[1]
rng = np.random.default_rng(127)
# Roots and open voicings for nylon strings.
HARMONY = (
    (43, 55, 62, 69), (47, 54, 59, 66), (48, 55, 64, 71), (50, 57, 62, 69),
    (45, 57, 64, 72), (48, 55, 60, 67), (43, 55, 62, 71), (50, 57, 64, 69),
    (40, 55, 59, 66), (47, 54, 62, 69), (48, 55, 60, 64), (43, 55, 62, 69),
    (45, 52, 60, 67), (48, 55, 64, 71), (50, 57, 62, 67), (50, 57, 64, 69),
    (43, 55, 59, 66), (45, 52, 60, 64), (48, 55, 64, 69), (47, 54, 59, 66),
    (45, 52, 60, 67), (48, 55, 60, 64), (50, 57, 62, 67), (50, 55, 62, 69),
)
FLUTE = (
    (1, 69, 1.8), (3.1, 71, 0.66), (4, 74, 2.6), (7.4, 71, 0.5),
    (9, 67, 2.3), (13, 69, 1.75), (15, 66, 0.8), (17.4, 64, 2.0),
    (21, 67, 2.0), (23.2, 71, 0.6), (24, 74, 2.1), (27, 71, 0.67),
    (29, 69, 2.15), (35, 66, 0.62), (36, 71, 1.3), (38, 69, 1.55),
    (41, 64, 2.0), (44, 67, 2.6), (49, 69, 2.0), (51.3, 72, 0.58),
    (53, 71, 1.65), (55, 67, 0.8), (57.1, 69, 2.4), (62, 66, 1.4),
    (65, 67, 1.7), (67.2, 71, 0.46), (68, 72, 2.2), (72, 76, 1.1),
    (73.6, 74, 1.48), (76, 71, 2.1), (81, 72, 1.7), (83.1, 71, 0.64),
    (85, 67, 2.3), (90, 66, 1.4), (92, 67, 1.9),
)


def frequency(note):
    return 440 * 2 ** ((note - 69) / 12)


def pluck(note):
    t = np.arange(round(2.8 * RATE)) / RATE
    bend = 0.002 * (1 - np.exp(-t * 19))
    phase = 2 * np.pi * frequency(note) * (t + bend)
    wave = np.zeros_like(t)
    for mode in range(1, 15):
        shape = np.sin(mode * 0.77) / mode ** 1.23
        ringing = np.exp(-t * (1.65 + mode ** 1.33 * 0.93))
        dispersion = 1 + mode * mode * 0.000062
        wave += shape * np.sin(phase * mode * dispersion) * ringing
    pick = sosfilt(butter(2, [380, 3100], fs=RATE, btype="bandpass", output="sos"),
                  rng.normal(size=len(t)))
    return (wave + 0.16 * pick * np.exp(-t * 93)) * np.minimum(t / 0.009, 1)


def flute(note, beats):
    length = beats * BEAT
    t = np.arange(round((length + 0.26) * RATE)) / RATE
    vibrato = 0.0023 * np.sin(2 * np.pi * 4.6 * t)
    bend = 1 - 0.012 * np.exp(-t * 20) + vibrato * np.minimum(t, 0.6)
    phase = np.cumsum(bend * (2 * np.pi * frequency(note) / RATE))
    air = sosfilt(butter(2, [650, 3400], fs=RATE, btype="bandpass", output="sos"),
                 rng.normal(size=len(t)))
    pressure = 0.87 + 0.10 * np.sin(2 * np.pi * 1.14 * t)
    tone = np.sin(phase) + 0.19 * np.sin(phase * 2.0 + 0.4) + 0.046 * np.sin(phase * 3)
    attack = np.clip(t / 0.14, 0, 1) ** 1.2
    release = np.clip((length + 0.22 - t) / 0.29, 0, 1) ** 1.7
    return (tone * 0.72 + air * 0.22) * pressure * attack * release


def compose():
    mix = np.zeros((round(len(HARMONY) * 4 * BEAT * RATE), 2))

    def add(wave, beats, level, pan):
        end = round(beats * BEAT * RATE) + np.arange(len(wave))
        mix[end % len(mix)] += wave[:, None] * level * np.sqrt([1 - pan, pan])

    for bar, voicing in enumerate(HARMONY):
        pattern = ((0, 0), (1.05, 1), (1.68, 2), (2.65, 3)) if bar % 4 != 3 else ((0, 0), (1.35, 2), (2.30, 1))
        for beat, string in pattern:
            start = bar * 4 + beat + rng.uniform(-0.023, 0.023)
            add(pluck(voicing[string]), start, (0.52 if string == 0 else 0.34) * rng.uniform(0.87, 1.1), 0.34)
        if bar in (3, 7, 10, 15, 19, 23):
            add(pluck(voicing[3] - 12), bar * 4 + 3.18, 0.18, 0.48)
    for start, note, length in FLUTE:
        add(flute(note, length), start, rng.uniform(0.15, 0.24), 0.59)

    # Quiet room reflections wrap with the performance for a continuous loop.
    dry = mix.copy()
    for delay, level in ((0.033, 0.115), (0.059, 0.10), (0.107, 0.069),
                         (0.193, 0.057), (0.317, 0.043), (0.449, 0.029)):
        mix += level * np.roll(dry[:, ::-1], round(RATE * delay), axis=0)
    mix = sosfilt(butter(2, [85, 6200], fs=RATE, btype="bandpass", output="sos"), mix, axis=0)
    return (mix * (0.61 / max(np.abs(mix).max(), 1e-6))).astype("<f4")


if __name__ == "__main__":
    output = ROOT / "assets" / "audio" / "grove.ogg"
    output.parent.mkdir(exist_ok=True, parents=True)
    subprocess.run([
        "ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-f", "f32le",
        "-ar", str(RATE), "-ac", "2", "-i", "-", "-c:a", "libvorbis", "-q:a", "4",
        "-metadata", "title=Between the Olives", str(output),
    ], input=compose().tobytes(), check=True)
