import { useWindowSize } from "@vueuse/core";

export default function Polygon(props: {
  points: [number, number][];
  fillColor?: string;
  strokeColor?: string;
  strokeWidth?: number;
}) {
  let { points, fillColor, strokeColor, strokeWidth } = props;

  fillColor ??= "rgba(34, 197, 94, 0.3)"; // Tailwind green-500 with opacity
  strokeColor ??= "rgba(34, 197, 94, 1)";
  strokeWidth ??= 2;

  if (points.length === 0) {
    return null;
  }

  // Find out the width and height of points.
  const minX = Math.min(...points.map((p) => p[0]));
  const maxX = Math.max(...points.map((p) => p[0]));

  const minY = Math.min(...points.map((p) => p[1]));
  const maxY = Math.max(...points.map((p) => p[1]));

  const width = maxX - minX;
  const height = maxY - minY;

  const size = useWindowSize();

  const screenWidth = size.width.value;
  const screenHeight = size.height.value;

  const scale =
    Math.min(
      Math.max(screenWidth / width, 1),
      Math.max(screenHeight / height, 1),
    ) / 1.6;

  console.log(screenWidth, width);

  const pathData =
    points
      .map((point, index) => {
        const [x, y] = point;
        return `${index === 0 ? "M" : "L"} ${x * scale} ${y * scale}`;
      })
      .join(" ") + " Z"; // Close the path

  // Tailwind CSS enabled.
  return (
    <div class="flex text-center">
      <svg class="absolute items-center w-full h-full pointer-events-none">
        <defs>
          <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="2" dy="2" stdDeviation="3" flood-opacity="0.3" />
          </filter>
        </defs>
        <path
          d={pathData}
          fill={fillColor}
          stroke={strokeColor}
          stroke-width={strokeWidth}
        />
      </svg>
    </div>
  );
}
