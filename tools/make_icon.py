"""Draws Ocular's app icon: an amber iris on a dark rounded square."""

import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

S = 1024
SS = 2
W = S * SS
CX = CY = W // 2

img = Image.new("RGBA", (W, W), (0, 0, 0, 0))

# Background: a dark vertical gradient inside a rounded square.
grad = Image.new("RGBA", (W, W))
top = (22, 25, 34)
bottom = (7, 8, 11)
draw = ImageDraw.Draw(grad)
for y in range(W):
    t = y / (W - 1)
    color = tuple(int(top[i] + (bottom[i] - top[i]) * t) for i in range(3)) + (255,)
    draw.line([(0, y), (W, y)], fill=color)

mask = Image.new("L", (W, W), 0)
ImageDraw.Draw(mask).rounded_rectangle(
    [0, 0, W - 1, W - 1], radius=int(W * 0.22), fill=255
)
img.paste(grad, (0, 0), mask)

# A faint inner edge so the icon reads as a tile on any wallpaper.
edge = Image.new("RGBA", (W, W), (0, 0, 0, 0))
ImageDraw.Draw(edge).rounded_rectangle(
    [int(W * 0.012), int(W * 0.012), W - int(W * 0.012), W - int(W * 0.012)],
    radius=int(W * 0.21),
    outline=(255, 255, 255, 30),
    width=max(2, int(W * 0.004)),
)
img = Image.alpha_composite(img, edge)

# Warm glow behind the iris.
glow = Image.new("RGBA", (W, W), (0, 0, 0, 0))
gd = ImageDraw.Draw(glow)
gr = int(W * 0.33)
gd.ellipse([CX - gr, CY - gr, CX + gr, CY + gr], fill=(245, 181, 68, 90))
img = Image.alpha_composite(img, glow.filter(ImageFilter.GaussianBlur(int(W * 0.05))))

d = ImageDraw.Draw(img)

# Outer ring and spokes.
R = int(W * 0.335)
d.ellipse(
    [CX - R, CY - R, CX + R, CY + R],
    outline=(245, 181, 68, 110),
    width=max(2, int(W * 0.004)),
)
for i in range(72):
    angle = 2 * math.pi * i / 72
    long = i % 6 == 0
    r1 = int(W * 0.236)
    r2 = int(W * (0.306 if long else 0.286))
    d.line(
        [
            CX + r1 * math.cos(angle),
            CY + r1 * math.sin(angle),
            CX + r2 * math.cos(angle),
            CY + r2 * math.sin(angle),
        ],
        fill=(245, 181, 68, 100 if long else 46),
        width=max(2, int(W * 0.0024)),
    )

# Iris: a radial amber gradient, painted outer to inner.
iris_radius = int(W * 0.19)
iris = Image.new("RGBA", (W, W), (0, 0, 0, 0))
idraw = ImageDraw.Draw(iris)
outer, mid, inner = (86, 48, 5), (240, 164, 55), (255, 233, 182)
steps = 240
for step in range(steps, 0, -1):
    radius = int(iris_radius * step / steps)
    t = step / steps
    if t > 0.5:
        k = (1 - t) / 0.5
        color = tuple(int(outer[i] + (mid[i] - outer[i]) * k) for i in range(3))
    else:
        k = t / 0.5
        color = tuple(int(mid[i] + (inner[i] - mid[i]) * k) for i in range(3))
    idraw.ellipse(
        [CX - radius, CY - radius, CX + radius, CY + radius],
        fill=color + (255,),
    )
img = Image.alpha_composite(img, iris)

d = ImageDraw.Draw(img)

# Pupil and a soft catch-light.
pupil = int(W * 0.086)
d.ellipse([CX - pupil, CY - pupil, CX + pupil, CY + pupil], fill=(8, 9, 12, 255))
d.ellipse(
    [CX - pupil, CY - pupil, CX + pupil, CY + pupil],
    outline=(245, 181, 68, 120),
    width=max(2, int(W * 0.003)),
)
hl = int(W * 0.017)
d.ellipse(
    [CX - int(W * 0.062) - hl, CY - int(W * 0.075) - hl, CX - int(W * 0.062) + hl, CY - int(W * 0.075) + hl],
    fill=(255, 240, 205, 165),
)

# Dashed rim, a nod to the launcher's iris.
rim = int(W * 0.322)
for i in range(96):
    if i % 2:
        continue
    start = 2 * math.pi * i / 96
    end = start + math.pi / 96
    d.arc(
        [CX - rim, CY - rim, CX + rim, CY + rim],
        start=math.degrees(start),
        end=math.degrees(end),
        fill=(255, 210, 122, 70),
        width=max(2, int(W * 0.003)),
    )

final = img.resize((S, S), Image.LANCZOS)
out = Path(__file__).resolve().parent.parent / "assets" / "icon.png"
out.parent.mkdir(parents=True, exist_ok=True)
final.save(out)
print(out)
