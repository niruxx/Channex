import { load, type Store } from '@tauri-apps/plugin-store'

let storePromise: Promise<Store> | null = null

export function getStore(): Promise<Store> {
  if (!storePromise) {
    storePromise = load('imageboarder-data.json', { autoSave: true })
  }
  return storePromise
}
