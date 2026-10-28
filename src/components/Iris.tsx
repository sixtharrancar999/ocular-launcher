import { useMemo } from "react";

/** A large, slow iris behind the Play page: rings, spokes and a glowing rim. */
export function Iris({ className }: { className?: string }) {
  const spokes = useMemo(
    () =>
      Array.from({ length: 72 }, (_, index) => {
        const angle = (index * 360) / 72;
        const long = index % 6 === 0;
        return (
          <line
            key={index}
            x1="300"
            y1={long ? 150 : 168}
            x2="300"
            y2={long ? 96 : 118}
            transform={`rotate(${angle} 300 300)`}
            stroke="#f5b544"
            strokeOpacity={long ? 0.28 : 0.14}
            strokeWidth={long ? 1.4 : 0.8}
          />
        );
      }),
    [],
  );

  return (
    <svg
      className={className}
      viewBox="0 0 600 600"
      fill="none"
      aria-hidden="true"
    >
      <defs>
        <radialGradient id="iris-glow" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor="#f5b544" stopOpacity="0.35" />
          <stop offset="55%" stopColor="#f5b544" stopOpacity="0.08" />
          <stop offset="100%" stopColor="#f5b544" stopOpacity="0" />
        </radialGradient>
        <radialGradient id="iris-core" cx="50%" cy="45%" r="60%">
          <stop offset="0%" stopColor="#ffe9b6" />
          <stop offset="40%" stopColor="#f0a437" />
          <stop offset="100%" stopColor="#5c3406" />
        </radialGradient>
        <linearGradient id="iris-rim" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stopColor="#ffd98a" stopOpacity="0.7" />
          <stop offset="50%" stopColor="#f5b544" stopOpacity="0.15" />
          <stop offset="100%" stopColor="#ffd98a" stopOpacity="0.55" />
        </linearGradient>
      </defs>

      <circle cx="300" cy="300" r="290" fill="url(#iris-glow)" />
      <g className="iris-spin">{spokes}</g>
      <circle cx="300" cy="300" r="210" stroke="url(#iris-rim)" strokeWidth="1.2" />
      <circle cx="300" cy="300" r="180" stroke="#f5b544" strokeOpacity="0.12" strokeWidth="1" strokeDasharray="2 6" />
      <circle cx="300" cy="300" r="150" stroke="#f5b544" strokeOpacity="0.18" strokeWidth="1" />
      <circle cx="300" cy="300" r="112" fill="url(#iris-core)" fillOpacity="0.16" stroke="#ffd98a" strokeOpacity="0.35" />
      <circle cx="300" cy="300" r="52" fill="#08090c" stroke="#f5b544" strokeOpacity="0.45" strokeWidth="1.5" />
      <circle cx="300" cy="300" r="52" fill="url(#iris-glow)" />
      <circle cx="281" cy="277" r="13" fill="#ffe9b6" fillOpacity="0.5" />
    </svg>
  );
}
