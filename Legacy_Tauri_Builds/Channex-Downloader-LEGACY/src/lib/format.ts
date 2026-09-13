export function formatBytes(bytes: number | null | undefined): string {
  if (bytes === null || bytes === undefined || Number.isNaN(bytes)) return "—";
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)));
  const value = bytes / 1024 ** i;
  return `${value >= 100 || i === 0 ? Math.round(value) : value.toFixed(1)} ${units[i]}`;
}

const MIME_BY_EXT: Record<string, string> = {
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  png: "image/png",
  gif: "image/gif",
  webp: "image/webp",
  bmp: "image/bmp",
  svg: "image/svg+xml",
  webm: "video/webm",
  mp4: "video/mp4",
  mov: "video/quicktime",
  avi: "video/x-msvideo",
  mkv: "video/x-matroska",
};

export function mimeForExt(ext: string): string {
  return MIME_BY_EXT[ext.trim().toLowerCase().replace(/^\./, "")] ?? "application/octet-stream";
}

export function dedupeFilenames<T extends { postNo: number; originalName: string }>(
  items: T[],
): Map<T, string> {
  const counts = new Map<string, number>();
  for (const item of items) {
    counts.set(item.originalName, (counts.get(item.originalName) ?? 0) + 1);
  }
  const result = new Map<T, string>();
  for (const item of items) {
    const isDup = (counts.get(item.originalName) ?? 0) > 1;
    result.set(item, isDup ? `${item.postNo}_${item.originalName}` : item.originalName);
  }
  return result;
}
