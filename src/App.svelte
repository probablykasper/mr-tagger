<script lang="ts">
  import { invoke, dialog } from '@tauri-apps/api'
  import { popup } from './scripts/helpers'
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
</script>

<main>
  <div class="sidebar">
    <div class="row">
      <button on:click={openDialog}>Open Files</button>
    </div>
    {#each openFiles as file, i}
      <div class="row" class:selected={i === selected} on:click={() => open(i)}
        >{file.replace(/^.*[\\\/]/, '')}</div>
    {/each}
    <FileDrop fileExtensions={['mp3']} handleFiles={addFiles} msg="" />
  </div>
  <div class="page">
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
  main
    display: flex
    color: #e6e6e6
    height: 100vh
  .page
    flex-grow: 1
  .sidebar
    position: relative
    width: 230px
    height: 100%
    background-color: rgba(#ffffff, 0.03)
    border-right: 1px solid rgba(#ffffff, 0.1)
    font-size: 13px
    .row
      padding: 8px
      cursor: default
    .row:nth-child(2n)
      background-color: rgba(#ffffff, 0.05)
    .row.selected
      background-color: #074a97
</style>
