<script lang="ts">
  import { invoke, dialog } from '@tauri-apps/api'
  import { checkShortcut, popup, runCmd } from './scripts/helpers'
  import PageView from './components/Item.svelte'
  import type { Page } from './components/Item.svelte'
  import FileDrop from './components/FileDrop.svelte'

  type File = {
    path: string
  }
  type App = {
    current_index: number
    files: File[]
  }
  let app: App = {
    current_index: 0,
    files: [],
  }
  ;(async () => {
    app = await runCmd<App>('open_files', { paths: [] })
  })()

  let page: Page | null = null
  $: if (app) getPage()
  async function getPage() {
    let newPage = await runCmd<Page>('get_page')
    if (!page || newPage.path !== page.path) {
      page = newPage
    }
  }

  async function openFiles(paths: string[]) {
    app = await runCmd<App>('open_files', { paths })
  }
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
      await openFiles(paths)
    }
  }
  async function show(index: number) {
    if (app.current_index !== index) {
      app = await runCmd<App>('show', { index })
    }
  }
  async function filesKeydown(e: KeyboardEvent) {
    if (checkShortcut(e, 'ArrowUp')) {
      e.preventDefault()
      if (app.current_index >= 1) {
        show(app.current_index - 1)
      }
    } else if (checkShortcut(e, 'ArrowDown')) {
      e.preventDefault()
      if (app.current_index < app.files.length - 1) {
        show(app.current_index + 1)
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
      {#each app.files as file, i}
        <div class="row" class:selected={i === app.current_index} on:click={() => show(i)}
          >{file.path.replace(/^.*[\\\/]/, '')}</div>
      {/each}
    </div>
    <FileDrop
      fileExtensions={['mp3', 'aiff', 'wav', 'm4a', 'mp4', 'm4p', 'm4b', 'm4r', 'm4v']}
      handleFiles={openFiles}
      msg="" />
  </div>
  <div class="main">
    {#if page}
      <PageView item={page} />
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
