export const LoadingSVG = ({
  diameter = 20,
  strokeWidth = 4,
}: {
  diameter?: number;
  strokeWidth?: number;
}) => (
  <svg
    className="animate-spin"
    fill="none"
    viewBox="0 0 24 24"
    style={{
      width: `${diameter}px`,
      height: `${diameter}px`,
    }}
  >
    <circle
      className="opacity-25"
      cx="12"
      cy="12"
      r="10"
      stroke="currentColor"
      strokeWidth={strokeWidth}
    ></circle>
    <path
      className="opacity-75"
      fill="currentColor"
      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
    ></path>
  </svg>
  // <span className="relative flex h-6 w-6 pointer-events-none border border-pink-800 rounded-full">
  //   <span className="animate-pulse absolute inline-flex h-full w-full rounded-full bg-pink-400 opacity-75"></span>
  // </span>
);
