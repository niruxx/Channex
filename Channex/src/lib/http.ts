import { fetch as tauriFetch } from '@tauri-apps/plugin-http'

export class HttpError extends Error {
  status: number
  constructor(status: number, message: string) {
    super(message)
    this.status = status
    this.name = 'HttpError'
  }
}

const DEFAULT_TIMEOUT = 20_000

export async function getJson<T>(url: string, opts?: { timeoutMs?: number }): Promise<T> {
  const controller = new AbortController()
  const timeout = setTimeout(() => controller.abort(), opts?.timeoutMs ?? DEFAULT_TIMEOUT)
  try {
    const res = await tauriFetch(url, {
      method: 'GET',
      signal: controller.signal,
      headers: { accept: 'application/json' },
    })
    if (!res.ok) {
      throw new HttpError(res.status, `${res.status} ${res.statusText} for ${url}`)
    }
    const text = await res.text()
    try {
      return JSON.parse(text) as T
    } catch {
      throw new HttpError(res.status, `Response from ${url} was not valid JSON (site may require a browser challenge)`)
    }
  } finally {
    clearTimeout(timeout)
  }
}

export async function getBytes(url: string, opts?: { timeoutMs?: number }): Promise<Uint8Array> {
  const controller = new AbortController()
  const timeout = setTimeout(() => controller.abort(), opts?.timeoutMs ?? 60_000)
  try {
    const res = await tauriFetch(url, { method: 'GET', signal: controller.signal })
    if (!res.ok) {
      throw new HttpError(res.status, `${res.status} ${res.statusText} for ${url}`)
    }
    const buf = await res.arrayBuffer()
    return new Uint8Array(buf)
  } finally {
    clearTimeout(timeout)
  }
}
