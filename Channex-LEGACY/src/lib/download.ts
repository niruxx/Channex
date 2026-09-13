import { open } from '@tauri-apps/plugin-dialog'
import { mkdir, writeFile, exists } from '@tauri-apps/plugin-fs'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { join } from '@tauri-apps/api/path'
import { getBytes } from './http'
import { sanitizeFileName } from './format'

export async function pickDownloadDir(defaultPath?: string): Promise<string | null> {
  const result = await open({ directory: true, multiple: false, defaultPath })
  if (!result) return null
  return Array.isArray(result) ? result[0] : result
}

export async function ensureDir(dir: string): Promise<void> {
  const already = await exists(dir).catch(() => false)
  if (!already) {
    await mkdir(dir, { recursive: true })
  }
}

export async function uniqueDestPath(dir: string, fileName: string): Promise<string> {
  const safe = sanitizeFileName(fileName)
  let candidate = await join(dir, safe)
  let i = 1
  const dot = safe.lastIndexOf('.')
  const base = dot > 0 ? safe.slice(0, dot) : safe
  const ext = dot > 0 ? safe.slice(dot) : ''
  while (await exists(candidate).catch(() => false)) {
    candidate = await join(dir, `${base} (${i})${ext}`)
    i++
  }
  return candidate
}

export async function downloadToFile(url: string, destPath: string): Promise<void> {
  const bytes = await getBytes(url)
  await writeFile(destPath, bytes)
}

export async function revealInFolder(path: string): Promise<void> {
  await revealItemInDir(path)
}
