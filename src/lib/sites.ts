import type { ChanSite } from '../types'

export const PRESET_SITES: ChanSite[] = [
  {
    id: '4chan',
    name: '4chan',
    schema: 'yotsuba',
    mediaLayout: 'flat-cdn',
    apiOrigin: 'https://a.4cdn.org',
    siteOrigin: 'https://boards.4chan.org',
    mediaOrigin: 'https://i.4cdn.org',
    accent: '#3fae6a',
    isCustom: false,
    nsfw: false,
    postEngine: 'external',
    defaultBoards: [
      { code: 'g', title: 'Technology' },
      { code: 'v', title: 'Video Games' },
      { code: 'a', title: 'Anime & Manga' },
      { code: 'tv', title: 'Television & Film' },
      { code: 'mu', title: 'Music' },
      { code: 'fit', title: 'Fitness' },
      { code: 'ck', title: 'Food & Cooking' },
      { code: 'diy', title: 'Do It Yourself' },
      { code: 'sci', title: 'Science & Math' },
      { code: 'lit', title: 'Literature' },
      { code: 'out', title: 'Outdoors' },
      { code: 'co', title: 'Comics & Cartoons' },
      { code: 'fa', title: 'Fashion' },
      { code: 'int', title: 'International' },
      { code: 'wg', title: 'Wallpapers' },
      { code: 'biz', title: 'Business & Finance' },
    ],
  },
  {
    id: '8kun',
    name: '8kun',
    schema: 'yotsuba',
    mediaLayout: 'file-store',
    apiOrigin: 'https://8kun.top',
    siteOrigin: 'https://8kun.top',
    mediaOrigin: 'https://media.8kun.top',
    accent: '#c9a227',
    isCustom: false,
    nsfw: true,
    postEngine: 'lynxchan',
    defaultBoards: [],
  },
  {
    id: '8chan-moe',
    name: '8chan.moe',
    schema: 'lynxchan',
    mediaLayout: 'board-dirs',
    apiOrigin: 'https://8chan.moe',
    siteOrigin: 'https://8chan.moe',
    mediaOrigin: 'https://8chan.moe',
    accent: '#8b5cf6',
    isCustom: false,
    nsfw: true,
    postEngine: 'lynxchan',
    defaultBoards: [],
  },
  {
    id: 'lainchan',
    name: 'Lainchan',
    schema: 'yotsuba',
    mediaLayout: 'board-dirs',
    apiOrigin: 'https://lainchan.org',
    siteOrigin: 'https://lainchan.org',
    mediaOrigin: 'https://lainchan.org',
    accent: '#4fc3c7',
    isCustom: false,
    nsfw: false,
    postEngine: 'external',
    defaultBoards: [],
  },
]

export function makeCustomSite(input: {
  id: string
  name: string
  schema: ChanSite['schema']
  mediaLayout: ChanSite['mediaLayout']
  origin: string
  mediaOrigin?: string
  accent?: string
  nsfw?: boolean
  postEngine?: ChanSite['postEngine']
}): ChanSite {
  const origin = input.origin.replace(/\/+$/, '')
  return {
    id: input.id,
    name: input.name,
    schema: input.schema,
    mediaLayout: input.mediaLayout,
    apiOrigin: origin,
    siteOrigin: origin,
    mediaOrigin: (input.mediaOrigin || origin).replace(/\/+$/, ''),
    accent: input.accent || '#6ee7c9',
    isCustom: true,
    nsfw: input.nsfw ?? true,
    postEngine: input.postEngine ?? 'external',
    defaultBoards: [],
  }
}
