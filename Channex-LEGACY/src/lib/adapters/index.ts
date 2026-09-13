import type { ChanAdapter, ChanSite } from '../../types'
import { yotsubaAdapter } from './yotsuba'
import { lynxchanAdapter } from './lynxchan'

export function getAdapter(site: ChanSite): ChanAdapter {
  switch (site.schema) {
    case 'lynxchan':
      return lynxchanAdapter
    case 'yotsuba':
    default:
      return yotsubaAdapter
  }
}
