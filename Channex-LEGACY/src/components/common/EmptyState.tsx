import { motion } from 'motion/react'
import type { LucideIcon } from 'lucide-react'
import { cn } from '../../lib/cn'

export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
  className,
}: {
  icon: LucideIcon
  title: string
  description?: string
  action?: React.ReactNode
  className?: string
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.35, ease: [0.16, 1, 0.3, 1] }}
      className={cn('flex h-full w-full flex-col items-center justify-center gap-4 px-6 text-center', className)}
    >
      <div
        className="animate-float flex size-18 items-center justify-center rounded-3xl border border-border-soft text-accent"
        style={{
          background: 'linear-gradient(150deg, color-mix(in oklab, var(--color-accent) 14%, var(--color-surface-2)), var(--color-surface-2))',
          boxShadow: 'var(--shadow-md)',
        }}
      >
        <Icon size={30} strokeWidth={1.5} />
      </div>
      <div className="space-y-1.5">
        <h3 className="text-[15px] font-semibold text-ink">{title}</h3>
        {description && <p className="max-w-sm text-sm leading-relaxed text-ink-faint">{description}</p>}
      </div>
      {action}
    </motion.div>
  )
}
