import { invoke } from '@tauri-apps/api/core';
import type { ProjectInfo } from '$lib/generated/ProjectInfo';
import type { RecentProjectListEntry } from '$lib/generated/RecentProjectListEntry';
import type { LaunchResult } from '$lib/generated/LaunchResult';

export async function createProject(
  folderPath: string,
  name: string,
  description?: string
): Promise<ProjectInfo> {
  return await invoke<ProjectInfo>('create_project', { folderPath, name, description });
}

export async function openProject(folderPath: string): Promise<LaunchResult> {
  return await invoke<LaunchResult>('open_project', { folderPath });
}

export async function closeProject(): Promise<void> {
  await invoke<void>('close_project');
}

export async function getCurrentProject(): Promise<ProjectInfo | null> {
  return await invoke<ProjectInfo | null>('get_current_project');
}

export async function checkLastOpenProject(): Promise<LaunchResult> {
  return await invoke<LaunchResult>('check_last_open_project_cmd');
}

export async function listAllProjects(): Promise<RecentProjectListEntry[]> {
  return await invoke<RecentProjectListEntry[]>('list_all_projects');
}

export async function removeRecentProject(folderPath: string): Promise<void> {
  await invoke<void>('remove_recent_project', { folderPath });
}

export async function deleteProject(folderPath: string): Promise<void> {
  await invoke<void>('delete_project', { folderPath });
}
