import { useId } from 'react'
import { motion } from 'motion/react'
import { cn } from '../../lib/cn'

export function BirthdayHat({ size = 20, rotate = -16, className }: { size?: number; rotate?: number; className?: string }) {
  const gradId = useId()

  return (
    <motion.svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      initial={{ opacity: 0, scale: 0.6, rotate: rotate - 10 }}
      animate={{ opacity: 1, scale: 1, rotate }}
      transition={{ type: 'spring', stiffness: 260, damping: 16 }}
      className={cn('pointer-events-none select-none drop-shadow', className)}
      aria-hidden="true"
    >
      <defs>
        <linearGradient id={gradId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#fb7185" />
          <stop offset="100%" stopColor="#8b5cf6" />
        </linearGradient>
      </defs>
      <ellipse cx="12" cy="19.6" rx="9.2" ry="1.6" fill="rgba(255,255,255,0.85)" />
      <path d="M12 1.5 L21 19.6 H3 Z" fill={`url(#${gradId})`} stroke="rgba(255,255,255,0.65)" strokeWidth="0.6" strokeLinejoin="round" />
      <circle cx="7.4" cy="14.4" r="1" fill="#fde68a" />
      <circle cx="12" cy="10.6" r="1" fill="#fde68a" />
      <circle cx="16.6" cy="14.4" r="1" fill="#fde68a" />
      <circle cx="12" cy="1.5" r="2.1" fill="#fde68a" />
    </motion.svg>
  )
}
