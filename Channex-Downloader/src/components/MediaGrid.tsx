import { AnimatePresence, motion } from "framer-motion";
import { ImageOff } from "lucide-react";
import { useShallow } from "zustand/react/shallow";
import { selectVisibleMedia, useAppStore } from "../store/useAppStore";
import { MediaCard } from "./MediaCard";

export function MediaGrid() {
  const visible = useAppStore(useShallow(selectVisibleMedia));

  if (visible.length === 0) {
    return (
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        className="flex flex-1 flex-col items-center justify-center gap-2 text-text-tertiary"
      >
        <ImageOff className="h-8 w-8" />
        <p className="text-sm">No media matches this filter</p>
      </motion.div>
    );
  }

  return (
    <motion.div
      layout
      className="grid grid-cols-3 gap-1 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8"
    >
      <AnimatePresence initial={false}>
        {visible.map((item, i) => (
          <MediaCard key={item.id} item={item} index={i} />
        ))}
      </AnimatePresence>
    </motion.div>
  );
}
