import { useEffect, useRef } from "react";

const SCALE = 7; // canvas pixels per skin texel
const WIDTH = 16; // skin units across
const HEIGHT = 34; // skin units down, with a one-unit margin above and below
const TOP = 1;

type Model = "classic" | "slim";

function paint(
  context: CanvasRenderingContext2D,
  texture: CanvasImageSource,
  source: [number, number, number, number],
  destination: [number, number, number, number],
) {
  context.drawImage(
    texture,
    source[0],
    source[1],
    source[2],
    source[3],
    destination[0],
    destination[1],
    destination[2],
    destination[3],
  );
}

/** Draws a front view of the skin, base layer first, then vanilla's overlay layer. */
function drawSkin(
  context: CanvasRenderingContext2D,
  texture: CanvasImageSource,
  model: Model,
) {
  context.clearRect(0, 0, WIDTH, HEIGHT);
  context.imageSmoothingEnabled = false;

  const slim = model === "slim";
  const armWidth = slim ? 3 : 4;
  const rightArmX = slim ? 1 : 0;
  const leftArmX = 12;
  const y = (unit: number) => unit + TOP;

  // Arms behind the torso, legs below it.
  paint(context, texture, [44, 20, armWidth, 12], [rightArmX, y(8), armWidth, 12]);
  paint(context, texture, [36, 52, armWidth, 12], [leftArmX, y(8), armWidth, 12]);
  paint(context, texture, [4, 20, 4, 12], [4, y(20), 4, 12]);
  paint(context, texture, [20, 52, 4, 12], [8, y(20), 4, 12]);

  // Torso.
  paint(context, texture, [20, 20, 8, 12], [4, y(8), 8, 12]);

  // Overlay layer: jacket, sleeves, pants and the hat, over their base parts.
  paint(context, texture, [44, 36, armWidth, 12], [rightArmX, y(8), armWidth, 12]);
  paint(context, texture, [52, 52, armWidth, 12], [leftArmX, y(8), armWidth, 12]);
  paint(context, texture, [0, 36, 4, 12], [4, y(20), 4, 12]);
  paint(context, texture, [0, 52, 4, 12], [8, y(20), 4, 12]);
  paint(context, texture, [20, 36, 8, 12], [4, y(8), 8, 12]);

  // Head last, with its hat layer on top.
  paint(context, texture, [8, 8, 8, 8], [4, y(0), 8, 8]);
  paint(context, texture, [40, 8, 8, 8], [4, y(0), 8, 8]);
}

export function FigurePreview({ src, slim }: { src: string; slim: boolean }) {
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const element = canvas.current;
    if (!element) {
      return;
    }
    const context = element.getContext("2d");
    if (!context) {
      return;
    }
    context.setTransform(1, 0, 0, 1, 0, 0);
    context.clearRect(0, 0, element.width, element.height);
    element.width = WIDTH * SCALE;
    element.height = HEIGHT * SCALE;
    context.setTransform(SCALE, 0, 0, SCALE, 0, 0);
    context.imageSmoothingEnabled = false;

    const texture = new Image();
    texture.onload = () => {
      const canvasElement = canvas.current;
      const drawing = canvasElement?.getContext("2d");
      if (!canvasElement || !drawing) {
        return;
      }
      drawing.setTransform(SCALE, 0, 0, SCALE, 0, 0);
      drawing.imageSmoothingEnabled = false;
      drawSkin(drawing, texture, slim ? "slim" : "classic");
    };
    texture.src = src;
    return () => {
      texture.onload = null;
    };
  }, [src, slim]);

  return (
    <canvas
      ref={canvas}
      className="skin-canvas"
      width={WIDTH * SCALE}
      height={HEIGHT * SCALE}
    />
  );
}

export function CapePreview({ src }: { src: string }) {
  return (
    <div className="cape-box">
      <img src={src} alt="" draggable={false} />
    </div>
  );
}
