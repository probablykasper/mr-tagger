<script lang="ts">
  import { dialog, event } from '@tauri-apps/api'
  import { checkShortcut, runCmd } from './scripts/helpers'
  import PageView from './components/Item.svelte'
  import type { Page } from './components/Item.svelte'
  import FileDrop from './components/FileDrop.svelte'
  import { onDestroy } from 'svelte'

  type File = {
    path: string
    dirty: boolean
  }
  type App = {
    current_index: number
    files: File[]
  }
  let app: App = {
    current_index: 0,
    files: [],
  }
  async function getApp() {
    app = await runCmd<App>('get_app')
  }
  getApp()

  let page: Page | null = null
  $: if (app) getPage()
  async function getPage() {
    let newPage = await runCmd<Page | null>('get_page')
    if (!page || !newPage || newPage.path !== page.path) {
      page = newPage
    }
  }

  async function openFiles(paths: string[]) {
    await runCmd<App>('open_files', { paths })
    getApp()
  }
  let extensions = ['mp3', 'aiff', 'wav', 'm4a', 'mp4', 'm4p', 'm4b', 'm4r', 'm4v']
  async function openDialog() {
    let paths = await dialog.open({
      filters: [{ name: 'Audio file', extensions }],
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
      await runCmd<App>('show', { index })
      getApp()
    }
  }
  async function close(index: number) {
    if (app.files[index].dirty) {
      let confirmed = await (window.confirm('Close without saving?') as any)
      if (!confirmed) return
    }
    await runCmd('close_file', { index })
    getApp()
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
  const unlistenFuture = event.listen('menu', ({ payload }) => {
    if (payload === 'Open...') {
      openDialog()
    }
  })
  onDestroy(async () => {
    const unlisten = await unlistenFuture
    unlisten()
  })
</script>

<main>
  <div class="sidebar">
    <div class="topbar">
      <button on:click={openDialog}>Open Files</button>
    </div>
    <div class="files" tabindex="0" on:keydown={filesKeydown}>
      {#each app.files as file, i}
        <div class="file" class:selected={i === app.current_index} on:click={() => show(i)}>
          <div class="icon x" on:click|stopPropagation={() => close(i)}>x</div>
          <div class="icon dirty">
            {#if file.dirty}
              <svg width="6" height="6" xmlns="http://www.w3.org/2000/svg">
                <circle cx="3" cy="3" r="2.5" />
              </svg>
            {/if}
          </div>
          {file.path.replace(/^.*[\\\/]/, '')}
        </div>
      {/each}
    </div>
    <FileDrop fileExtensions={extensions} handleFiles={openFiles} msg="" />
  </div>
  <div class="main">
    {#if page}
      <PageView item={page} on:appRefresh={getApp} />
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
    .file
      display: flex
      align-items: center
      padding: 7px 8px
      padding-left: 0px
      cursor: default
      .icon
        display: flex
        align-items: center
        justify-content: center
        width: 6px
        min-width: 6px
        margin-left: 6px
        margin-right: 4px
      .x
        display: none
      &:hover .x
        display: block
      &:hover .dirty
        display: none
      .dirty svg
        fill: #ffffff
    .file:nth-child(2n)
      background-color: rgba(#ffffff, 0.05)
    .file.selected
      background-color: hsl(147, 0%, 35%)
    &:focus .file.selected
      background-color: hsl(147, 70%, 30%)
</style>
