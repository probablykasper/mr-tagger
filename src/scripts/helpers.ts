import { invoke } from '@tauri-apps/api'

export function popup(msg: string) {
  invoke('error_popup', { msg })
}
