export function isMac(): boolean {
  if (typeof navigator === 'undefined') return false
  const platform = (navigator as any).userAgentData?.platform ?? navigator.platform ?? navigator.userAgent
  return /mac/i.test(platform)
}
