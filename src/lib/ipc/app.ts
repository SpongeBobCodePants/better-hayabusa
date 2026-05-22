import { invoke } from '@tauri-apps/api/core';
import type { AppVersion } from '$lib/generated/AppVersion';

export async function getAppVersion(): Promise<AppVersion> {
  return await invoke<AppVersion>('get_app_version');
}

export async function getAppState(key: string): Promise<string | null> {
  return await invoke<string | null>('get_app_state', { key });
}

export async function setAppState(key: string, value: string): Promise<void> {
  await invoke<void>('set_app_state', { key, value });
}

// Typed convenience wrappers for the M2 settings.

export type LaunchBehavior = 'last_project' | 'home_page';
// Any IANA timezone name (e.g. 'America/New_York') plus the sentinels
// 'UTC' and 'Local'. Validated on read in getDefaultTimezone.
export type TimezoneMode = string;

export async function getLaunchBehavior(): Promise<LaunchBehavior> {
  const v = await getAppState('launch_behavior');
  return v === 'home_page' ? 'home_page' : 'last_project';
}

export async function setLaunchBehavior(v: LaunchBehavior): Promise<void> {
  await setAppState('launch_behavior', v);
}

export async function getDefaultTimezone(): Promise<TimezoneMode> {
  const v = await getAppState('default_timezone');
  if (v === null) return 'UTC';
  if (v === 'UTC' || v === 'Local') return v;
  try {
    const supported = Intl.supportedValuesOf('timeZone');
    if (supported.includes(v)) return v;
  } catch {
    // Intl.supportedValuesOf not available - accept the stored value as-is.
    return v;
  }
  return 'UTC';
}

export async function setDefaultTimezone(v: TimezoneMode): Promise<void> {
  await setAppState('default_timezone', v);
}

export async function getRecentProjectsCount(): Promise<number> {
  const v = await getAppState('recent_projects_count');
  if (v === null) return 5;
  const n = parseInt(v, 10);
  return Number.isFinite(n) && n >= 1 && n <= 50 ? n : 5;
}

export async function setRecentProjectsCount(n: number): Promise<void> {
  await setAppState('recent_projects_count', String(n));
}
