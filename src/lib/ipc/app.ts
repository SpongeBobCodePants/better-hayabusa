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
