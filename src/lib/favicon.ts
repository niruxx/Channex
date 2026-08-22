import type { ChanSite } from '../types'

export function faviconCandidates(site: ChanSite): string[] {
  let hostname: string
  try {
    hostname = new URL(site.siteOrigin).hostname
  } catch {
    return []
  }
  return [
    `${site.siteOrigin}/favicon.ico`,
    `https://www.google.com/s2/favicons?sz=128&domain=${hostname}`,
  ]
}
