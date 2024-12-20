import struct
import pandas as pd

# Calculate for every 64-element window how many duplicates
def dupes_histo(window_size, xs):
    dupes = []
    for i in range(len(xs) - window_size):
        window = xs[i:i+window_size]
        dupe_count = len(window) - len(set(window))
        dupes.append(dupe_count)
    return dupes


with open("/Users/aduffy/Downloads/sample_floats.bin", "rb") as f:
    xs = []
    for i in range(66304):
        xs.append(struct.unpack("f", f.read(4))[0])

assert len(xs) == 66304


for WINDOW_SIZE in [64, 128, 1024]:
    dupes = pd.Series(dupes_histo(WINDOW_SIZE, xs))

    print(f"Analysis for WINDOW_SIZE {WINDOW_SIZE}:")

    print("- # windows: {}".format(len(dupes)))
    print("- min dupes in any window: {}".format(dupes.min()))
    print("- max dupes in any window: {}".format(dupes.max()))
    print("- # windows w/max dupes: {}".format(len(dupes[dupes == dupes.max()])))
    print("- average dupes per window: {}".format(dupes.mean()))
