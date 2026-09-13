import { create } from 'zustand'
import type { PostFile } from '../types'

interface LightboxState {
  files: PostFile[]
  index: number
  isOpen: boolean
  open: (files: PostFile[], index: number) => void
  close: () => void
  next: () => void
  prev: () => void
}

export const useLightboxStore = create<LightboxState>((set) => ({
  files: [],
  index: 0,
  isOpen: false,
  open: (files, index) => set({ files, index, isOpen: true }),
  close: () => set({ isOpen: false }),
  next: () => set((s) => ({ index: (s.index + 1) % Math.max(s.files.length, 1) })),
  prev: () => set((s) => ({ index: (s.index - 1 + s.files.length) % Math.max(s.files.length, 1) })),
}))
