import DOMPurify from 'dompurify'

let hooked = false

function ensureHooks() {
  if (hooked) return
  hooked = true
  DOMPurify.addHook('afterSanitizeAttributes', (node) => {
    if (node.tagName === 'A') {
      const href = node.getAttribute('href') || ''
      const cls = node.getAttribute('class') || ''
      const isQuote = /quotelink|quote(?!$)/.test(cls) || href.startsWith('#p')
      if (isQuote) {
        const match = /#p?(\d+)/.exec(href) || /(\d+)$/.exec(href)
        if (match) node.setAttribute('data-quote-id', match[1])
        node.setAttribute('data-quotelink', '1')
        node.setAttribute('href', '#')
      } else {
        node.setAttribute('target', '_blank')
        node.setAttribute('rel', 'noopener noreferrer')
        node.setAttribute('data-extlink', href)
      }
    }
  })
}

const ALLOWED_TAGS = [
  'a', 'b', 'strong', 'i', 'em', 'u', 's', 'del', 'span', 'br', 'p',
  'pre', 'code', 'blockquote', 'sub', 'sup', 'small', 'wbr', 'ul', 'ol', 'li',
]

const ALLOWED_ATTR = ['href', 'class', 'title']

export function sanitizeCommentHtml(raw: string): string {
  ensureHooks()
  return DOMPurify.sanitize(raw ?? '', {
    ALLOWED_TAGS,
    ALLOWED_ATTR,
    ALLOW_DATA_ATTR: false,
  })
}

export function decodeEntities(text: string): string {
  return text
    .replace(/&gt;/g, '>')
    .replace(/&lt;/g, '<')
    .replace(/&quot;/g, '"')
    .replace(/&#0?39;/g, "'")
    .replace(/&amp;/g, '&')
}

export function htmlToText(html: string): string {
  const clean = DOMPurify.sanitize(html ?? '', { ALLOWED_TAGS: ['br', 'p'] })
  return decodeEntities(
    clean
      .replace(/<br\s*\/?>/gi, '\n')
      .replace(/<\/p>/gi, '\n')
      .replace(/<[^>]+>/g, ''),
  ).trim()
}
