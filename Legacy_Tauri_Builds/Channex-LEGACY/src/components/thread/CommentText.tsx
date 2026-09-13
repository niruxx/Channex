import { openUrl } from '@tauri-apps/plugin-opener'

export function CommentText({
  html,
  onQuoteClick,
}: {
  html: string
  onQuoteClick?: (postId: string) => void
}) {
  function handleClick(e: React.MouseEvent<HTMLDivElement>) {
    const target = e.target as HTMLElement
    const quote = target.closest<HTMLElement>('[data-quotelink]')
    if (quote) {
      e.preventDefault()
      const id = quote.getAttribute('data-quote-id')
      if (id && onQuoteClick) onQuoteClick(id)
      return
    }
    const ext = target.closest<HTMLElement>('[data-extlink]')
    if (ext) {
      e.preventDefault()
      const url = ext.getAttribute('data-extlink')
      if (url) openUrl(url).catch(() => {})
    }
  }

  return (
    <div className="chan-comment selectable text-sm text-ink" onClick={handleClick} dangerouslySetInnerHTML={{ __html: html }} />
  )
}
