import { writable, type Readable } from 'svelte/store';
import type { ProjectInfo } from '$lib/generated/ProjectInfo';
import {
  closeProject as ipcClose,
  createProject as ipcCreate,
  getCurrentProject as ipcGet,
  openProject as ipcOpen,
} from '$lib/ipc/projects';

const { subscribe, set } = writable<ProjectInfo | null>(null);

export const currentProject: Readable<ProjectInfo | null> = { subscribe };

/** Hydrate the store from Rust on app boot. */
export async function loadCurrentProject(): Promise<void> {
  set(await ipcGet());
}

/** Create + auto-install as current. */
export async function createAndInstall(
  folderPath: string,
  name: string,
  description?: string
): Promise<ProjectInfo> {
  const info = await ipcCreate(folderPath, name, description);
  set(info);
  return info;
}

/** Open + auto-install as current. Returns the LaunchResult so callers
 *  can branch on schema-too-new etc. */
export async function openAndInstall(folderPath: string) {
  const result = await ipcOpen(folderPath);
  if (result.kind === 'Loaded') {
    set(result.info);
  }
  return result;
}

/** Close + clear store. */
export async function close(): Promise<void> {
  await ipcClose();
  set(null);
}
