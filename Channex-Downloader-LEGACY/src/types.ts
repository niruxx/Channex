export type Site = "4chan" | "lynxchan" | "vichan";

export interface MediaItem {
  id: string;
  postNo: number;
  originalName: string;
  ext: string;
  mediaUrl: string;
  thumbUrl: string;
  width: number | null;
  height: number | null;
  sizeBytes: number | null;
  isVideo: boolean;
}

export interface ThreadInfo {
  site: Site;
  origin: string;
  board: string;
  threadNo: number;
  subject: string | null;
  opComment: string | null;
  replyCount: number;
  media: MediaItem[];
}

export type MediaFilter = "all" | "images" | "videos";

export type DownloadStatus =
  | "queued"
  | "downloading"
  | "done"
  | "error"
  | "cancelled";

export interface DownloadEntry {
  status: DownloadStatus;
  downloaded: number;
  total: number | null;
  error?: string;
}

export type PreferredSiteId = "4chan" | "8chan_moe" | "8chan_se" | "custom";

export interface AppSettings {
  preferredSite: PreferredSiteId;
  customOrigin: string | null;
  downloadRoot: string | null;
  concurrency: number;
}

export interface SitePreset {
  id: PreferredSiteId;
  label: string;
  origin: string | null;
  example: string;
}

export const SITE_PRESETS: SitePreset[] = [
  {
    id: "4chan",
    label: "4chan",
    origin: "https://boards.4chan.org",
    example: "https://boards.4chan.org/g/thread/12345678",
  },
  {
    id: "8chan_moe",
    label: "8chan.moe",
    origin: "https://8chan.moe",
    example: "https://8chan.moe/tech/res/12345.html",
  },
  {
    id: "8chan_se",
    label: "8chan.se",
    origin: "https://8chan.se",
    example: "https://8chan.se/tech/res/12345.html",
  },
  {
    id: "custom",
    label: "Custom / other",
    origin: null,
    example: "https://your-imageboard.example/b/res/123.html",
  },
];
