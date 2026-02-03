# Convert height data from: https://www.geodaten.sachsen.de/
# to a height map PNG image
from PIL import Image
import sys
import os

if len(sys.argv) != 2:
    print("Usage: xyz_to_png.py <input.xyz>")
    sys.exit(1)

input_file = sys.argv[1]

if not input_file.lower().endswith(".xyz"):
    print("Error: input file must have .xyz extension")
    sys.exit(1)

output_file = os.path.splitext(input_file)[0] + ".png"

# --- read data ---
points = []

with open(input_file, "r") as f:
    for line in f:
        if not line.strip():
            continue
        x, y, z = map(float, line.split())
        points.append((x, y, z))

# --- build coordinate grids ---
xs = sorted(set(p[0] for p in points))
ys = sorted(set(p[1] for p in points))

width = len(xs)
height = len(ys)

x_index = {x: i for i, x in enumerate(xs)}
y_index = {y: i for i, y in enumerate(ys)}

# --- find Z range ---
z_values = [p[2] for p in points]
z_min = min(z_values)
z_max = max(z_values)
z_range = z_max - z_min or 1.0

print(f"Image size: {width}x{height}, Z range: {z_min} to {z_max}")

z_norm = 255 / z_range
print(f"Z normalization factor: {z_norm} levels per meter")

# --- create image ---
img = Image.new("L", (width, height))  # L = 8-bit grayscale
pixels = img.load()

# --- fill pixels ---
for x, y, z in points:
    px = x_index[x]
    py = y_index[y]

    # normalize Z to 0–255
    value = int((z - z_min) * z_norm)

    # flip Y so origin is bottom-left
    pixels[px, height - 1 - py] = value

# --- save ---
img.save(output_file)
print(f"Saved {output_file}")
