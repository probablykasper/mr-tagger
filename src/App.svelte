<script lang="ts">
  import { dialog, event } from '@tauri-apps/api'
  import { checkShortcut, runCmd } from './scripts/helpers'
  import PageView from './components/Page.svelte'
  import type { Page } from './components/Page.svelte'
  import { onDestroy } from 'svelte'
  import FileDrop from 'svelte-tauri-filedrop'
  import { fade } from 'svelte/transition'

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
  let extensions = [
    //
    'aiff',
    'mp3',
    'm4a',
    'mp4',
    'm4p',
    'm4b',
    'm4r',
    'm4v',
    'opus',
    'wav',
  ]
  async function openDialog() {
    let paths = await dialog.open({
      filters: [{ name: 'Audio/Video file', extensions }],
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
      let confirmed = window.confirm('Close without saving?')
      if (!confirmed) return
    }
    await runCmd('close_file', { index })
    getApp()
  }
  async function saveFile(saveAs: boolean) {
    await runCmd('save_file', { index: app.current_index, saveAs })
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
  const unlistenFuture = event.listen('menu', async ({ payload }) => {
    if (payload === 'Open...') {
      openDialog()
    } else if (payload === 'Close') {
      if (app.files.length === 0) {
        await runCmd('close_window')
      } else {
        close(app.current_index)
      }
    } else if (payload === 'Save') {
      saveFile(false)
    } else if (payload === 'Save As...') {
      saveFile(true)
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
          <div class="icon dirty">
            {#if file.dirty}
              <svg width="6" height="6" xmlns="http://www.w3.org/2000/svg">
                <circle cx="3" cy="3" r="2.5" />
              </svg>
            {/if}
          </div>
          <div class="icon x" on:click|stopPropagation={() => close(i)}>
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"
              ><path
                d="M23.954 21.03l-9.184-9.095 9.092-9.174-2.832-2.807-9.09 9.179-9.176-9.088-2.81 2.81 9.186 9.105-9.095 9.184 2.81 2.81 9.112-9.192 9.18 9.1z"
              /></svg
            >
          </div>
          {file.path.replace(/^.*[\\/]/, '')}
        </div>
      {/each}
    </div>
    <FileDrop {extensions} handleFiles={openFiles} let:files>
      {#if files.length > 0}
        <div class="dropzone" transition:fade={{ duration: 100 }} />
      {/if}
    </FileDrop>
  </div>
  <div class="main">
    {#if page}
      <button on:click={() => saveFile(false)} tabindex="0">Save</button>
      <PageView {page} on:appRefresh={getApp} />
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
    display: flex
    position: relative
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
      padding: 6px 8px
      padding-left: 0px
      cursor: default
      .icon
        display: flex
        align-items: center
        justify-content: center
        width: 8px
        height: 8px
        flex-shrink: 0
        padding: 4px
        margin: 0px 5px
        border-radius: 2px
        transition: opacity 180ms ease-out, transform 180ms ease-out
        &:hover
          background-color: rgba(#ffffff, 0.15)
      .icon.x
        opacity: 0
        transform: scale(0.5)
        position: absolute
      &:hover .icon.x
        display: flex
        opacity: 1
        transform: scale(1)
      &:hover .icon.dirty
        opacity: 0
        transform: scale(0.5)
      svg
        fill: #ffffff
    .file:nth-child(2n)
      background-color: rgba(#ffffff, 0.05)
    .file.selected
      background-color: hsl(147, 0%, 35%)
    &:focus .file.selected
      background-color: hsl(147, 70%, 30%)
      background-color: #103fcb
  .dropzone
    position: absolute
    width: 100%
    height: 100%
    top: 0px
    left: 0px
    background-color: rgba(#ffffff, 0.2)
</style>
