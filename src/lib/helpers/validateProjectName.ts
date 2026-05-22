/**
 * Mirror of src-tauri/src/project/name.rs. Keep in sync — backend
 * re-validates so the user can't bypass this by editing the IPC call.
 *
 * Returns null if the name is valid, or a human-readable error message.
 */
export function validateProjectName(name: string): string | null {
  const trimmed = name.trim();
  if (trimmed.length === 0) return 'Name cannot be empty.';
  if ([...trimmed].length > 100) return 'Name is too long (max 100 characters).';

  const forbidden = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
  for (const c of trimmed) {
    if (forbidden.includes(c)) return `Name cannot contain '${c}'.`;
    if (c.charCodeAt(0) < 0x20) return 'Name cannot contain control characters.';
  }
  if (trimmed.endsWith('.') || trimmed.endsWith(' ')) {
    return 'Name cannot end with a dot or space.';
  }
  const reserved = new Set([
    'CON', 'PRN', 'AUX', 'NUL',
    'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9',
    'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9',
  ]);
  const base = trimmed.split('.')[0].toUpperCase();
  if (reserved.has(base)) return `'${trimmed}' is a Windows-reserved name.`;
  return null;
}

/**
 * Mirror of validate_project_description in src-tauri/src/project/name.rs.
 * Returns null if valid, or a human-readable error message.
 *
 * Uses spread for code-point counting so emojis and combining characters
 * count as 1, matching Rust's `chars().count()`.
 */
export function validateProjectDescription(desc: string): string | null {
  if ([...desc].length > 250) {
    return 'Description is too long (max 250 characters).';
  }
  return null;
}
