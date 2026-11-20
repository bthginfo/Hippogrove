"""Render Hippogrove's original island scores with GeneralUser GS 2.0.3.

Usage: python art/build_music.py /path/to/GeneralUser-GS.sf2
Requires FluidSynth, pyFluidSynth, NumPy and FFmpeg. The bank is a build input.
"""
import argparse
from dataclasses import dataclass
from pathlib import Path
import random
import subprocess

import fluidsynth
import numpy as np

RATE = 44100
OUT = Path(__file__).resolve().parents[1] / "assets" / "audio"

# Bass, inner voices and open upper string for each harmony, in D major / B minor.
CHORDS = {
    "D": (38, 57, 62, 66), "G": (43, 55, 59, 66), "Bm": (47, 54, 62, 66),
    "A": (45, 57, 61, 64), "Em": (40, 55, 59, 64), "F#m": (42, 54, 61, 69),
    "D/F#": (42, 57, 62, 66), "G/A": (45, 55, 59, 62),
}
# Two-bar phrases. Rests are written into the lines. Winds and keys trade the answer.
PHRASES = [
    [(0.5, 66, 1.25), (2, 69, .68), (3.1, 71, 1.65), (5, 69, .62)],
    [(0, 66, 1.5), (1.8, 64, .8), (3, 62, 2.35)],
    [(1, 62, .65), (2, 66, .8), (3, 69, 1.45), (4.8, 73, .72)],
    [(0, 71, 1.35), (1.65, 69, .75), (3, 66, 2.)],
    [(0, 67, 1.65), (2.1, 69, .55), (3, 71, 2.4)],
    [(0.5, 73, 1.), (2, 74, .8), (3.25, 69, 1.8)],
    [(0.5, 67, 1.2), (2.25, 66, .55), (3, 64, 1.8), (5, 61, .55)],
    [(0, 62, 2.4)],
]


@dataclass(frozen=True)
class Score:
    filename: str
    title: str
    tempo: int
    transpose: int
    programs: tuple
    harmony: str
    reply: tuple


VALLEY = "D G D/F# A Bm G Em A G D/F# Em Bm G A D A"
MINOR = "Bm F#m G D/F# Em Bm G/A A G D/F# Em Bm G A D D"
SHORE = "G D/F# Em A D G Em A Bm G D/F# A Em G/A D D"
SCORES = (
    Score("cove", "Die kleine Bucht", 77, 0, (46, 42, 0, 71, 45),
          "D G Bm A D/F# G Em A G D/F# Em Bm G A D D", (0, 1, 2, 3)),
    Score("grove", "Wege übers Wasser", 89, 0, (46, 42, 0, 71, 45),
          VALLEY + " " + MINOR, (0, 1, 2, 3)),
    Score("fjord", "Unter den Kiefern", 72, -2, (24, 42, 46, 69, 48),
          MINOR + " " + SHORE, (4, 3, 6, 7)),
    Score("terraces", "Licht auf den Stufen", 83, 3, (107, 43, 46, 75, 12),
          SHORE + " " + VALLEY, (2, 7, 4, 1)),
    Score("lake", "Boote im Schilf", 97, -5, (12, 32, 98, 71, 108),
          SHORE + " " + MINOR, (0, 3, 4, 7)),
    Score("aegean", "Die helle Mole", 94, 2, (25, 43, 24, 68, 45),
          VALLEY + " " + SHORE, (5, 3, 2, 7)),
)


def compose(bank, score):
    tutorial = score.filename == "cove"
    bars = score.harmony.split()
    beat = 60 / score.tempo
    # Give the shore, weather and workers a few bars of their own before the reprise.
    length = round((len(bars) + (2 if tutorial else 4)) * 3 * beat * RATE)
    rng = random.Random(score.title)
    events = []

    def note(channel, pitch, start, duration, velocity):
        pitch += score.transpose
        start = max(0, start + rng.uniform(-.018, .018))
        velocity = max(1, min(127, velocity + rng.randint(-3, 3)))
        events.append((round(start * beat * RATE), channel, pitch, velocity))
        events.append((round((start + duration) * beat * RATE), channel, pitch, 0))

    for bar, symbol in enumerate(bars):
        bass, low, mid, high = CHORDS[symbol]
        start = 3 * bar
        # The harp leaves beats free for gameplay sounds and the answering phrase.
        note(0, bass + 12, start, 1.65, 57)
        if bar % 4 != 3:
            note(0, mid, start + 1.04, .95, 47)
            note(0, high, start + (2.46 if bar % 2 else 1.62), 1.15, 43)
        if score.filename == "lake" and bar % 4 != 3:
            note(4, high - 12, start + .55, .33, 57)
            note(0, low, start + 2.55, .45, 48)
        elif score.filename == "aegean" and bar % 2 == 0:
            note(0, high, start + 2.67, .28, 48)
        elif score.filename == "fjord" and bar % 4 == 0:
            note(4, mid, start, 4.6, 42)
        if not tutorial:
            note(1, bass, start + .05, 2.55 if bar % 2 else 1.9, 59)
            if bar >= 8 and bar % 4 != 3:
                note(4, low, start + 1., .58, 49)
                note(4, mid, start + 2.06, .64, 43)
        elif bar in (4, 8, 12):
            note(1, bass + 12, start, 4.5, 44)
        if bar % 2 == 0:
            phrase = bar // 2 % len(PHRASES)
            if bar >= 16:
                phrase = score.reply[(bar // 2) % 4]
            voice = 2 if tutorial or (bar // 4) % 2 else 3
            for offset, pitch, duration in PHRASES[phrase]:
                if tutorial and bar in (4, 10):
                    continue
                if not tutorial and 16 <= bar < 24:
                    pitch -= 12
                note(voice, pitch, start + offset, duration, 59 if voice == 2 else 74)

    synth = fluidsynth.Synth(gain=.7, samplerate=RATE)
    try:
        sfid = synth.sfload(str(bank))
        if sfid < 0:
            raise ValueError(f"Could not open instrument bank {bank}")
        for channel, (pan, level) in enumerate([(48, 97), (56, 78), (69, 96), (74, 81), (80, 73)]):
            synth.program_select(channel, sfid, 0, score.programs[channel])
            for controller, value in [(10, pan), (7, level), (91, 36), (93, 5)]:
                synth.cc(channel, controller, value)
        # Render the tail past the downbeat and fold it into the head of the loop.
        mix = np.zeros((length + RATE * 4, 2), dtype=np.float32)
        cursor = 0
        for frame, channel, pitch, velocity in sorted(events):
            if frame > cursor:
                mix[cursor:frame] = synth.get_samples(frame - cursor).reshape(-1, 2)
            if velocity:
                synth.noteon(channel, pitch, velocity)
            else:
                synth.noteoff(channel, pitch)
            cursor = frame
        mix[cursor:] = synth.get_samples(len(mix) - cursor).reshape(-1, 2)
    finally:
        synth.delete()
    tail = mix[length:]
    tail *= np.linspace(1., 0., len(tail), dtype=np.float32)[:, None]
    mix[:len(tail)] += tail
    mix = mix[:length] / 32768.
    # Keep perceived level consistent between arrangements while retaining their dynamics.
    rms = np.sqrt(np.mean(mix ** 2))
    mix *= min(.118 / rms, .78 / np.max(np.abs(mix)))
    return mix.astype("<f4")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bank", type=Path)
    bank = parser.parse_args().bank
    for score in SCORES:
        subprocess.run([
            "ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-f", "f32le",
            "-ar", str(RATE), "-ac", "2", "-i", "-", "-c:a", "libvorbis", "-q:a", "4",
            "-metadata", f"title={score.title}", str(OUT / f"{score.filename}.ogg"),
        ], input=compose(bank, score).tobytes(), check=True)
