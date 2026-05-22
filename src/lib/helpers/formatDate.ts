import { getDefaultTimezone, type TimezoneMode } from '$lib/ipc/app';

/**
 * Formats a UTC ISO 8601 timestamp string per the user's default_timezone
 * setting. Cached per-render where the caller uses the same mode.
 *
 * Async to allow reading the setting from app.db; in hot paths, callers
 * should fetch the mode once via getDefaultTimezone and call formatDateSync.
 */
export async function formatDate(isoUtc: string): Promise<string> {
  const mode = await getDefaultTimezone();
  return formatDateSync(isoUtc, mode);
}

export function formatDateSync(isoUtc: string, mode: TimezoneMode): string {
  const d = new Date(isoUtc);
  if (isNaN(d.getTime())) return isoUtc; // fall back to raw string

  if (mode === 'UTC') {
    // YYYY-MM-DD HH:MM:SS UTC
    const iso = d.toISOString();
    return `${iso.slice(0, 10)} ${iso.slice(11, 19)} UTC`;
  }

  const opts: Intl.DateTimeFormatOptions = {
    year: 'numeric', month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit', second: '2-digit',
    hour12: false,
  };
  if (mode !== 'Local') {
    opts.timeZone = mode;
  }
  try {
    return new Intl.DateTimeFormat(undefined, opts).format(d);
  } catch {
    // Invalid timezone - fall back to Local-style format without a timeZone.
    delete opts.timeZone;
    return new Intl.DateTimeFormat(undefined, opts).format(d);
  }
}
