import { invoke } from '@tauri-apps/api'
import type { event } from '@tauri-apps/api'

export function popup(msg: string) {
  invoke('error_popup', { msg })
}

export function extractUnlistener(futureUnlistener: Promise<event.UnlistenFn>) {
  return async () => {
    const unlisten = await futureUnlistener
    unlisten()
  }
}
