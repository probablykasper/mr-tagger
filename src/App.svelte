<script lang="ts">
  import { invoke, dialog } from '@tauri-apps/api'
  import { checkShortcut, popup } from './scripts/helpers'
  import ItemView from './components/Item.svelte'
  import type { Item } from './components/Item.svelte'
  import FileDrop from './components/FileDrop.svelte'

  let openFiles: string[] = []
  let selected: number | null = null
  async function addFiles(files: string[]) {
    for (const file of files) {
      if (openFiles.includes(file)) {
        popup('Skipping duplicate file: ' + file)
      } else {
        openFiles.push(file)
        openFiles = openFiles
      }
    }
    if (selected === null && openFiles.length > 0) {
      open(openFiles.length - 1)
    }
  }
  let item: Item | null = null
  async function openDialog() {
    let paths = await dialog.open({
      filters: [{ name: 'Audio file', extensions: ['mp3', 'm4a', 'wav', 'aiff'] }],
      multiple: true,
      directory: false,
    })
    if (typeof paths === 'string') {
      paths = [paths]
    }
    if (paths !== null) {
      await addFiles(paths)
    }
  }
  async function open(index: number) {
    item = (await invoke('open', { path: openFiles[index] }).catch(popup)) as any
    selected = index
  }
  async function filesKeydown(e: KeyboardEvent) {
    if (checkShortcut(e, 'ArrowUp')) {
      e.preventDefault()
      if (selected !== null && selected > 0) {
        open(selected - 1)
      } else if (selected === null && openFiles.length >= 1) {
        open(openFiles.length - 1)
      }
    } else if (checkShortcut(e, 'ArrowDown')) {
      e.preventDefault()
      if (selected !== null && selected < openFiles.length - 1) {
        open(selected + 1)
      } else if (selected === null && openFiles.length >= 1) {
        open(0)
      }
    }
  }
</script>

<main>
  <div class="sidebar">
    <div class="topbar">
      <button on:click={openDialog}>Open Files</button>
    </div>
    <div class="files" tabindex="0" on:keydown={filesKeydown}>
      {#each openFiles as file, i}
        <div class="row" class:selected={i === selected} on:click={() => open(i)}
          >{file.replace(/^.*[\\\/]/, '')}</div>
      {/each}
    </div>
    <FileDrop
      fileExtensions={['mp3', 'aiff', 'wav', 'm4a', 'mp4', 'm4p', 'm4b', 'm4r', 'm4v']}
      handleFiles={addFiles}
      msg="" />
  </div>
  <div class="main">
    {#if item}
      <ItemView {item} />
    {/if}
  </div>
</main>

<style lang="sass">
  :global(body)
    margin: 0
    font-family: Arial, Helvetica, sans-serif
    font-size: 18px
    background-color: #191B20
    overflow: hidden
    user-select: none
    -webkit-user-select: none
  main
    display: flex
    color: #e6e6e6
    height: 100vh
  .main
    flex-grow: 1
    width: 0px
    overflow: auto
  .sidebar
    position: relative
    display: flex
    flex-direction: column
    width: 250px
    height: 100%
    background-color: #202227
    border-right: 1px solid rgba(#ffffff, 0.1)
    font-size: 12px
  .topbar
    padding: 8px
    border-bottom: 1px solid rgba(#ffffff, 0.1)
  .files
    overflow-y: auto
    height: 100%
    outline: none
    .row
      padding: 7px 8px
      cursor: default
    .row:nth-child(2n)
      background-color: rgba(#ffffff, 0.05)
    .row.selected
      background-color: hsl(147, 0%, 35%)
    &:focus .row.selected
      background-color: hsl(147, 70%, 30%)
</style>
