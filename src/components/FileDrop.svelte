<script lang="ts">
  import { fade } from 'svelte/transition'
  import { extractUnlistener } from '../scripts/helpers'
  import { event } from '@tauri-apps/api'
  import { onMount } from 'svelte'

  export let msg = 'Drop files'
  let droppable = false
  export let fileExtensions: string[] = []
  export let handleFiles: (files: string[]) => void

  // workaround for https://github.com/tauri-apps/tauri/issues/2323
  let readyToListen = false
  setTimeout(() => (readyToListen = true), 100)

  function getValidPaths(paths: string[]) {
    let validPaths = []
    for (const path of paths) {
      for (const ext of fileExtensions) {
        if (path.endsWith('.' + ext)) {
          validPaths.push(path)
        }
      }
    }
    return validPaths
  }
  onMount(() => {
    const unlisten = event.listen('tauri://file-drop-hover', (e) => {
      if (!readyToListen) return
      const validPaths = getValidPaths(e.payload as string[])
      if (validPaths.length > 0) {
        droppable = true
      }
    })
    return extractUnlistener(unlisten)
  })
  onMount(() => {
    const unlisten = event.listen('tauri://file-drop', (e) => {
      if (!readyToListen) return
      const validPaths = getValidPaths(e.payload as string[])
      if (validPaths.length > 0) {
        droppable = false
      }
      handleFiles(validPaths)
    })
    return extractUnlistener(unlisten)
  })
  onMount(() => {
    const unlisten = event.listen('tauri://file-drop-cancelled', (e) => {
      if (!readyToListen) return
      droppable = false
    })
    return extractUnlistener(unlisten)
  })
</script>

{#if droppable}
  <!-- if the overlay is always visible, it's not possible to scroll while dragging tracks -->
  <div class="drag-overlay" transition:fade={{ duration: 100 }}>
    <h1>{msg}</h1>
  </div>
  <div class="dropzone" />
{/if}

<style lang="sass">
  .dropzone, .drag-overlay
    position: absolute
    width: 100%
    height: 100%
    top: 0px
    left: 0px
  .drag-overlay
    display: flex
    align-items: center
    justify-content: center
    background-color: rgba(#ffffff, 0.2)
    transition: all 100ms ease-in-out
</style>
