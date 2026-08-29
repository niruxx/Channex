import { motion } from "framer-motion";
import { ImagesIcon } from "lucide-react";
import { useAppStore } from "../store/useAppStore";

export function EmptyState() {
  const error = useAppStore((s) => s.error);
  const loading = useAppStore((s) => s.loading);

  if (loading) {
    return (
      <div className="grid flex-1 grid-cols-3 content-start gap-1 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8">
        {Array.from({ length: 24 }).map((_, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: i * 0.015 }}
            className="animate-shimmer aspect-square bg-surface-hover"
          />
        ))}
      </div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
      className="flex flex-1 flex-col items-center justify-center gap-3 text-center"
    >
      <div className="flex h-16 w-16 items-center justify-center rounded-full bg-accent-100 text-accent-500">
        <ImagesIcon className="h-8 w-8" />
      </div>
      <h2 className="text-lg font-medium text-text-primary">Paste a thread link to get started</h2>
      <p className="max-w-sm text-sm text-text-secondary">
        Works with 4chan, and most lynxchan &amp; vichan based imageboards. Every image and
        video in the thread will show up here as a preview grid you can bulk download.
      </p>
      {error && (
        <motion.p
          initial={{ opacity: 0, scale: 0.97 }}
          animate={{ opacity: 1, scale: 1 }}
          className="mt-2 max-w-md whitespace-pre-line rounded-lg border border-danger-500/30 bg-danger-100 px-4 py-2 text-sm text-danger-600"
        >
          {error}
        </motion.p>
      )}
    </motion.div>
  );
}
