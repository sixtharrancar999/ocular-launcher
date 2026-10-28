interface PatchProps {
  src: string;
  x: number;
  y: number;
  w: number;
  h: number;
  className?: string;
}

const TEXTURE = 64;

/** One face of a skin texture, taken from vanilla's 64×64 layout. */
export function SkinFace({ src, x, y, w, h, className }: PatchProps) {
  const position = (offset: number, size: number) => {
    const remaining = TEXTURE - size;
    if (remaining <= 0) {
      return 0;
    }
    return (offset / remaining) * 100;
  };
  return (
    <span
      className={`skin-patch ${className ?? ""}`}
      style={{
        backgroundImage: `url("${src}")`,
        backgroundSize: `${(TEXTURE / w) * 100}% ${(TEXTURE / h) * 100}%`,
        backgroundPosition: `${position(x, w)}% ${position(y, h)}%`,
      }}
    />
  );
}

/** Head: the front face, plus the top of the head read through the same box. */
export function HeadPreview({ src }: { src: string }) {
  return (
    <div className="face-box">
      <SkinFace src={src} x={8} y={8} w={8} h={8} />
      <span
        className="skin-patch face-top"
        style={{
          backgroundImage: `url("${src}")`,
          backgroundSize: "800% 800%",
          backgroundPosition: `${(8 / 56) * 100}% 0%`,
        }}
      />
    </div>
  );
}

/** Flat figure: head, torso, two arms and two legs, each face from the texture. */
export function FigurePreview({ src, slim }: { src: string; slim: boolean }) {
  return (
    <div className={`figure ${slim ? "slim" : "classic"}`}>
      <SkinFace src={src} x={8} y={8} w={8} h={8} className="figure-head" />
      <SkinFace src={src} x={44} y={20} w={4} h={12} className="figure-arm left" />
      <SkinFace src={src} x={20} y={20} w={8} h={12} className="figure-torso" />
      <SkinFace src={src} x={36} y={52} w={4} h={12} className="figure-arm right" />
      <SkinFace src={src} x={4} y={20} w={4} h={12} className="figure-leg left" />
      <SkinFace src={src} x={20} y={52} w={4} h={12} className="figure-leg right" />
    </div>
  );
}

export function CapePreview({ src }: { src: string }) {
  return (
    <div className="cape-box">
      <img src={src} alt="" draggable={false} />
    </div>
  );
}
