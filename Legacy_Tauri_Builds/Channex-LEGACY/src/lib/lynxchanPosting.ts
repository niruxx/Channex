import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import type { ChanSite, ReplyDraft } from '../types'
import { getBytes } from './http'
import { uid } from './format'

export interface CaptchaHandle {
  imageUrl: string
  fetchedAt: number
}

export async function fetchCaptcha(site: ChanSite, boardCode: string): Promise<CaptchaHandle> {
  const bust = uid()
  const bytes = await getBytes(`${site.apiOrigin}/captcha.js?boardUri=${encodeURIComponent(boardCode)}&d=${bust}`)
  const blob = new Blob([bytes.buffer as ArrayBuffer], { type: 'image/png' })
  return { imageUrl: URL.createObjectURL(blob), fetchedAt: Date.now() }
}

export type PostResult = { ok: true; id: number } | { ok: false; error: string }

async function submit(url: string, fields: Record<string, string | undefined>, file?: File): Promise<PostResult> {
  const form = new FormData()
  for (const [key, value] of Object.entries(fields)) {
    if (value !== undefined && value !== '') form.append(key, value)
  }
  if (file) form.append('files', file, file.name)

  try {
    const res = await tauriFetch(url, { method: 'POST', body: form })
    const text = await res.text()
    let json: any
    try {
      json = JSON.parse(text)
    } catch {
      return { ok: false, error: `Unexpected response from server (HTTP ${res.status}). The site may be unavailable or its posting form changed.` }
    }
    if (!res.ok || (json.status && json.status !== 'ok')) {
      return { ok: false, error: typeof json.data === 'string' ? json.data : json.status || 'Post was rejected.' }
    }
    return { ok: true, id: typeof json.data === 'number' ? json.data : 0 }
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) }
  }
}

export async function postReply(site: ChanSite, boardCode: string, threadId: string, draft: ReplyDraft): Promise<PostResult> {
  return submit(
    `${site.apiOrigin}/replyThread.js?json=1`,
    {
      boardUri: boardCode,
      threadId,
      message: draft.comment,
      name: draft.name,
      email: draft.options,
      captcha: draft.captchaAnswer,
      spoiler: draft.spoiler ? '1' : undefined,
      password: sessionPassword(),
    },
    draft.file,
  )
}

let cachedPassword: string | null = null
function sessionPassword(): string {
  if (!cachedPassword) cachedPassword = Math.random().toString(36).slice(2, 10)
  return cachedPassword
}
