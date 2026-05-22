/**
 * Timezone catalog helpers.
 *
 * Exposes the full list of selectable timezones (UTC, Local, every IANA zone)
 * with each zone's current UTC offset annotated. The list is built lazily on
 * first call and cached for the lifetime of the module.
 */

export type TimezoneOption = { value: string; label: string };

let cachedList: TimezoneOption[] | null = null;

const LOCAL_LABEL = 'Local (browser timezone)';

/**
 * Returns the current UTC longOffset (e.g. "GMT-05:00") for an IANA zone
 * using Intl.DateTimeFormat. Returns the raw offset string from `formatToParts`
 * (typically prefixed `GMT`) or an empty string if the zone can't be resolved.
 */
function getLongOffset(zone: string): string {
  try {
    const fmt = new Intl.DateTimeFormat('en', {
      timeZone: zone,
      timeZoneName: 'longOffset',
    });
    const parts = fmt.formatToParts(new Date());
    const tzPart = parts.find((p) => p.type === 'timeZoneName');
    return tzPart?.value ?? '';
  } catch {
    return '';
  }
}

/**
 * Builds a human-friendly label for a timezone value.
 * - 'UTC' -> 'UTC'
 * - 'Local' -> the local-browser-timezone label
 * - any IANA zone -> e.g. 'America/New_York (UTC-05:00)' (with a real minus
 *   sign when the offset is negative, matching `longOffset`'s output).
 */
export function formatTimezoneLabel(zone: string): string {
  if (zone === 'UTC') return 'UTC';
  if (zone === 'Local') return LOCAL_LABEL;
  const offset = getLongOffset(zone);
  // longOffset is typically "GMT-05:00" or "GMT+00:00"; rewrite GMT -> UTC.
  const display = offset.startsWith('GMT') ? `UTC${offset.slice(3)}` : offset;
  return display ? `${zone} (${display})` : zone;
}

/**
 * Returns the full list of selectable timezones, sorted as:
 *   1. UTC
 *   2. Local
 *   3. every IANA zone, alphabetically
 *
 * Each entry's label includes the current UTC offset annotation.
 * Built once and cached.
 */
export function getAllTimezones(): TimezoneOption[] {
  if (cachedList !== null) return cachedList;

  const list: TimezoneOption[] = [
    { value: 'UTC', label: 'UTC' },
    { value: 'Local', label: LOCAL_LABEL },
  ];

  let zones: string[] = [];
  try {
    zones = Intl.supportedValuesOf('timeZone');
  } catch {
    zones = [];
  }

  zones.sort((a, b) => a.localeCompare(b));
  // Dedupe against entries already seeded above (UTC and Local) so a
  // runtime where Intl.supportedValuesOf includes 'UTC' doesn't yield
  // duplicate Svelte keys on the picker.
  const seen = new Set(list.map((o) => o.value));
  for (const zone of zones) {
    if (seen.has(zone)) continue;
    seen.add(zone);
    list.push({ value: zone, label: formatTimezoneLabel(zone) });
  }

  cachedList = list;
  return list;
}
