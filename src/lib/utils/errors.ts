/**
 * Sanitize error messages for user display.
 * Strips file paths, stack traces, and internal details.
 */
export function sanitizeError(e: unknown): string {
  const raw = e instanceof Error ? e.message : String(e);
  return raw
    .replace(/\/[\w./\-]+/g, "[path]")
    .replace(/at\s+.*$/gm, "")
    .trim();
}
