function sanitizeSegment(segment: string): string {
  return segment.replace(/[<>:"/\\|?*\x00-\x1f]/g, "_").trim() || "_";
}

function timestampSlug(d = new Date()): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}_${pad(d.getHours())}${pad(d.getMinutes())}${pad(d.getSeconds())}`;
}

export function joinPath(...parts: string[]): string {
  return parts
    .map((p) => p.trim())
    .filter(Boolean)
    .map((p, i) => (i === 0 ? p.replace(/[/\\]+$/, "") : p.replace(/^[/\\]+|[/\\]+$/g, "")))
    .join("/");
}

/** Builds the `/{board}/{timestamp}_{board}_{threadNo}` folder under a root. */
export function computeThreadDownloadPath(root: string, board: string, threadNo: number): string {
  const safeBoard = sanitizeSegment(board);
  const folder = `${timestampSlug()}_${safeBoard}_${threadNo}`;
  return joinPath(root, safeBoard, folder);
}
