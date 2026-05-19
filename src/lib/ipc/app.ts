import { invoke } from '@tauri-apps/api/core';
import type { AppVersion } from '$lib/generated/AppVersion';

export async function getAppVersion(): Promise<AppVersion> {
  return await invoke<AppVersion>('get_app_version');
}
