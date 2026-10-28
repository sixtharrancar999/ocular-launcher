"""Makes the demo skins used while developing Ocular's skin page.

Run:  python tools/make_demo_skins.py
Writes assets/demo/skin-classic.png and assets/demo/cape.png (64x64 and 64x32 PNG).
"""

from pathlib import Path

from PIL import Image

OUT = Path(__file__).resolve().parent.parent / "assets" / "demo"
OUT.mkdir(parents=True, exist_ok=True)

SKIN_W, SKIN_H = 64, 64
skin = Image.new("RGBA", (SKIN_W, SKIN_H), (0, 0, 0, 0))
pixels = skin.load()

SKIN = (212, 168, 132, 255)
SKIN_SHADE = (186, 143, 110, 255)
HAIR = (46, 34, 28, 255)
SHIRT = (38, 46, 74, 255)
SHIRT_DARK = (28, 34, 56, 255)
SHIRT_SLEEVE = (46, 56, 88, 255)
PANTS = (44, 48, 58, 255)
SHOES = (28, 30, 36, 255)
EYES = (24, 22, 28, 255)
AMBER = (240, 164, 55, 255)


def fill(x0, y0, x1, y1, colour):
    for y in range(y0, y1):
        for x in range(x0, x1):
            pixels[x, y] = colour


# --- head (8x8 faces) -----------------------------------------------------------------------
fill(0, 0, 32, 16, SKIN)
# Face: hair on the top, a fringe and eyes.
fill(8, 8, 16, 11, HAIR)
fill(8, 8, 10, 13, HAIR)
fill(14, 8, 16, 13, HAIR)
fill(9, 12, 11, 13, EYES)
fill(13, 12, 15, 13, EYES)
fill(10, 15, 14, 16, SKIN_SHADE)
# Back and sides: all hair, with the iris on the head-top.
fill(0, 8, 8, 16, HAIR)
fill(16, 8, 24, 16, HAIR)
fill(24, 8, 32, 16, HAIR)
fill(14, 1, 18, 3, AMBER)

# --- body -----------------------------------------------------------------------------------
fill(16, 16, 40, 32, SHIRT)
# Torso front: a lighter collar; back: plain.
fill(20, 16, 28, 18, SHIRT_SLEEVE)
fill(22, 18, 26, 22, AMBER)

# --- right arm (0..16 x 16..32), left arm (32..48) -------------------------------------------
fill(40, 16, 56, 32, SHIRT_SLEEVE)
fill(40, 28, 56, 32, SKIN)
fill(32, 16, 48, 32, SHIRT_SLEEVE)
fill(32, 28, 48, 32, SKIN)

# --- legs (0..16 y32..64) and (16..32) -------------------------------------------------------
fill(0, 32, 32, 48, PANTS)
fill(0, 48, 32, 52, SHOES)
fill(0, 48, 32, 64, PANTS)
fill(8, 48, 16, 52, SHOES)
fill(24, 48, 32, 52, SHOES)

skin.save(OUT / "skin-classic.png")

# --- cape 64x32: front half only, amber with the Ocular ring --------------------------------
cape = Image.new("RGBA", (64, 32), (0, 0, 0, 0))
cp = cape.load()

BODY_W, BODY_H = 10, 16
for y in range(BODY_H):
    for x in range(BODY_W):
        cp[x, y] = (26, 22, 18, 255) if (x + y) % 2 == 0 else (34, 28, 22, 255)

cx, cy, radius = 5, 8, 4
for y in range(BODY_H):
    for x in range(BODY_W):
        distance = ((x + 0.5 - cx) ** 2 + (y + 0.5 - cy) ** 2) ** 0.5
        if radius - 0.6 <= distance <= radius + 0.2:
            cp[x, y] = AMBER
        elif distance < radius - 1.6:
            cp[x, y] = (40, 32, 22, 255)

cape.save(OUT / "cape.png")
print(OUT / "skin-classic.png")
print(OUT / "cape.png")
