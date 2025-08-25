export default function Polygon(props: {
  points: [number, number][];
  fillColor?: string;
  strokeColor?: string;
  strokeWidth?: number;
}) {
  let { points, fillColor, strokeColor, strokeWidth } = props;

  fillColor ??= 'rgba(34, 197, 94, 0.3)'; // Tailwind green-500 with opacity
  strokeColor ??= 'rgba(34, 197, 94, 1)';
  strokeWidth ??= 2;

  if (points.length === 0) {
    return null;
  }

  const pathData = points
    .map((point, index) => {
      const [x, y] = point;
      return `${index === 0 ? 'M' : 'L'} ${x * 3} ${y * 3}`;
    })
    .join(' ') + ' Z'; // Close the path

  // Tailwind CSS enabled.
  return (
    <svg class="absolute top-0 left-0 w-full h-full pointer-events-none">
      <defs>
        <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
          <feDropShadow dx="2" dy="2" stdDeviation="3" floodOpacity="0.3"/>
        </filter>
      </defs>
      <path d={pathData} fill={fillColor} stroke={strokeColor} stroke-width={strokeWidth} />
    </svg>
  );
}