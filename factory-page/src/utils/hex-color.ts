export function u32toHex(value: number) {
  return value.toString(16).padStart(6, "0").toUpperCase();
}

/**
 * Parse a color from a string.
 *
 * Supports:
 * - u32 number
 * - hex string
 * - #RRGGBB
 * - RRGGBB
 */
export function parseColor(value: string): number {
  // Try to parse as a hex string
  if (value.startsWith("#")) {
    value = value.slice(1);
  }
  if (value.length === 6 || value.length === 3) {
    return parseInt(value, 16);
  }
  // Try to parse as a u32 number
  return parseInt(value, 10);
}
