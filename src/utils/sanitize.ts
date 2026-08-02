/**
 * Sanitize a project name into a safe folder name.
 *
 * Mirrors the backend `sanitize_folder_name` in `src-tauri/src/commands.rs`:
 * trim whitespace, replace control characters and reserved path characters
 * (`<> : " / \\ | ? * .`) with `_`, and replace spaces with `_`.
 */
export function sanitizeFolderName(name: string): string {
  return name
    .trim()
    .replace(/[\x00-\x1f<>:"/\\|?*.]/g, '_')
    .replace(/ /g, '_')
}
