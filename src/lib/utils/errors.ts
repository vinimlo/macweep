/**
 * Sanitize error messages for user display.
 * Strips file paths, stack traces, and internal details.
 */
export function sanitizeError(e: unknown): string {
	if (e instanceof Error) {
		// Strip file paths and stack traces
		return e.message
			.replace(/\/[\w./\-]+/g, '[path]')
			.replace(/at\s+.*$/gm, '')
			.trim();
	}

	const str = String(e);
	return str
		.replace(/\/[\w./\-]+/g, '[path]')
		.replace(/at\s+.*$/gm, '')
		.trim();
}
