import { ImageIcon, MessageSquare, Video } from "lucide-react";
import { useAppStore } from "../store/useAppStore";

const SITE_LABEL: Record<string, string> = {
  "4chan": "4chan",
  lynxchan: "LynxChan",
  vichan: "vichan",
};

export function ThreadHeader() {
  const thread = useAppStore((s) => s.thread);
  if (!thread) return null;

  const imageCount = thread.media.filter((m) => !m.isVideo).length;
  const videoCount = thread.media.filter((m) => m.isVideo).length;

  return (
    <div className="animate-fade-in-up flex flex-col gap-1.5 pb-4">
      <div className="flex flex-wrap items-center gap-2">
        <span className="rounded-full bg-accent-100 px-2.5 py-0.5 text-xs font-medium text-accent-600">
          {SITE_LABEL[thread.site] ?? thread.site}
        </span>
        <span className="text-sm text-text-secondary">
          /{thread.board}/ &middot; thread {thread.threadNo}
        </span>
      </div>
      {thread.subject && (
        <h2 className="truncate text-lg font-medium text-text-primary">{thread.subject}</h2>
      )}
      <div className="flex items-center gap-4 text-sm text-text-secondary">
        <span className="flex items-center gap-1.5">
          <MessageSquare className="h-4 w-4" /> {thread.replyCount} replies
        </span>
        <span className="flex items-center gap-1.5">
          <ImageIcon className="h-4 w-4" /> {imageCount}
        </span>
        <span className="flex items-center gap-1.5">
          <Video className="h-4 w-4" /> {videoCount}
        </span>
      </div>
    </div>
  );
}
