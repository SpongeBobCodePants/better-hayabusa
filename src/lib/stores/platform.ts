import { platform as tauriPlatform } from '@tauri-apps/plugin-os';

export type SupportedPlatform = 'windows' | 'macos' | 'linux' | 'other';

let detected: SupportedPlatform | null = null;

export async function detectPlatform(): Promise<SupportedPlatform> {
  if (detected !== null) return detected;
  const raw = await tauriPlatform();
  detected = raw === 'windows' || raw === 'macos' || raw === 'linux' ? raw : 'other';
  return detected;
}
