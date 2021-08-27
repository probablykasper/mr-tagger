<script context="module" lang="ts">
  export type Frame = {
    Text?: { id: string; value: string }
  }
  export type Image = {
    index: number
    total_images: number
    data: Uint8Array
    mime_type: string
    description: string | null
    picture_type: string | null
  }
  export type Comment = {
    lang: string | null
    description: string | null
    text: string
  }
  export type Page = {
    path: string
    title: string
    artists: string[]
    album: string
    album_artists: string[]
    composer: string[]
    groupings: string[]
    genres: string[]
    track_num: number
    track_total: number
    disc_num: number
    disc_total: number
    compilation: boolean
    bpm: string
    comments: Comment[]
    frames: Frame[]
  }
  let x = 0 // to fix syntax highlighting
</script>

<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import { runCmd } from '../scripts/helpers'
  import { dialog } from '@tauri-apps/api'
  import FileDrop from './FileDrop.svelte'
  import MultiField from './MultiField.svelte'

  export let page: Page

  let image: Image | null = null
  $: if (page) {
    image = null
    getImage(null)
  }
  async function getImage(index: number | null) {
    image = (await runCmd('get_image', { index })) as Image | null
  }

  const dispatch = createEventDispatcher()
  async function removeImage() {
    if (image) {
      await runCmd('remove_image', { index: image.index })
      getImage(null)
      dispatch('appRefresh')
    }
  }
  async function setImage(path?: string) {
    if (!path) {
      let pathResult = await dialog.open({
        filters: [{ name: 'Audio file', extensions: ['jpg', 'jpeg', 'png', 'bmp'] }],
        multiple: false,
        directory: false,
      })
      if (typeof pathResult === 'string') path = pathResult
    }
    if (image) {
      await runCmd('set_image', { index: image.index, path })
      getImage(image.index)
    } else {
      await runCmd('set_image', { index: 0, path })
      getImage(0)
    }
    dispatch('appRefresh')
  }
  let showFrames = true
  let svgWidth = 0
</script>

<main>
  <div class="left">
    <div class="cover">
      {#if image}
        <img src={'data:' + image.mime_type + ';base64,' + image.data} alt="" />
      {:else}
        <div class="svg-cover" bind:clientWidth={svgWidth} style={'height:' + svgWidth + 'px'}>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            preserveAspectRatio="xMidYMin meet"
            viewBox="0 0 24 24">
            <path
              d="M23 0l-15.996 3.585v13.04c-2.979-.589-6.004 1.671-6.004 4.154 0 2.137 1.671 3.221 3.485 3.221 2.155 0 4.512-1.528 4.515-4.638v-10.9l12-2.459v8.624c-2.975-.587-6 1.664-6 4.141 0 2.143 1.715 3.232 3.521 3.232 2.14 0 4.476-1.526 4.479-4.636v-17.364z" />
          </svg>
        </div>
      {/if}
      <FileDrop fileExtensions={['jpeg', 'jpg', 'png', 'bmp']} handleOneFile={setImage} msg="" />
    </div>
    {#if image}
      <div>
        <button on:click={removeImage}>Remove</button>
        <button on:click={() => setImage()}>Replace</button>
      </div>
      <div class="text">{image.index + 1} of {image.total_images}</div>
      <div class="text">{image.mime_type}</div>
      {#if image.picture_type}
        <div class="text">Type: {image.picture_type}</div>
      {/if}
      {#if image.description}
        <div class="text">Description: {image.description}</div>
      {/if}
    {:else}
      <div>
        <button on:click={() => setImage()}>Add</button>
      </div>
    {/if}
  </div>
  <div class="right">
    <div class="row">
      <span class="label">Path</span>
      <span class="content">{page.path}</span>
    </div>
    <div class="row">
      <span class="label">Title</span>
      <span class="content">{page.title}</span>
    </div>
    <div class="row">
      <span class="label">Artist</span>
      <MultiField value={page.artists} />
    </div>
    <div class="row">
      <span class="label">Album</span>
      <span class="content">{page.album}</span>
    </div>
    <div class="row">
      <span class="label">Album artist</span>
      <MultiField value={page.album_artists} />
    </div>
    <div class="row">
      <span class="label">Composer</span>
      <MultiField value={page.composer} />
    </div>
    <div class="row">
      <span class="label">Grouping</span>
      <MultiField value={page.groupings} />
    </div>
    <div class="row">
      <span class="label">Genre</span>
      <MultiField value={page.genres} />
    </div>
    <div class="row">
      <span class="label">Track</span>
      <div class="content">{page.track_num || '_'} of {page.track_num || '_'}</div>
    </div>
    <div class="row">
      <span class="label">Disc number</span>
      <div class="content">{page.disc_num || '_'} of {page.disc_num || '_'}</div>
    </div>
    <div class="row">
      <span class="label">Compilation</span>
      <div class="content">{page.compilation ? 'Yes' : 'No'}</div>
    </div>
    <div class="row">
      <span class="label">BPM</span>
      <div class="content">{page.bpm}</div>
    </div>
    <div class="row">
      <span class="label">Comments</span>
      <div>
        {#each page.comments as comment}
          <div class="content comment">
            {#if comment.lang !== null}
              <br />
              <br />
              Lang: {comment.lang}
              <br />
            {/if}
            {#if comment.description !== null}
              Description: {comment.description}
              <br />
              <br />
            {/if}
            {comment.text}
          </div>
        {/each}
      </div>
    </div>
    <button class="toggle" tabindex="0" on:click={() => (showFrames = !showFrames)}
      >{showFrames ? 'Hide tags' : 'Show tags'}</button>
    <div class="frames">
      {#if showFrames}
        {#each page.frames as frame}
          {#if frame.Text}
            <div class="frame-label">{frame.Text.id}</div>
            <div class="content">{frame.Text.value}</div>
          {/if}
        {/each}
      {/if}
    </div>
  </div>
</main>

<style lang="sass">
  main
    display: flex
    font-size: 13px
    padding-bottom: 12px
    padding-right: 12px
  .text
    user-select: auto
    -webkit-user-select: auto
  .left
    min-width: 160px
    width: calc(50% - 160px)
    max-width: 250px
    padding-left: 12px
  img
    display: block
    width: 100%
    min-height: 80px
    object-fit: contain
  .cover
    position: relative
  .svg-cover
    padding: 28%
    box-sizing: border-box
    background-color: #2b2c31
  svg
    box-sizing: border-box
    fill: #45464a
  .right
    width: 0px
    flex-grow: 1
  .row
    padding: 5px 0px
    display: flex
    align-items: baseline
    .content
      display: contents // safari selection fix
  .content
    font-size: 13px
    user-select: auto
    -webkit-user-select: auto
  .comment
    display: block
  .label
    display: inline-block
    width: 76px
    flex-shrink: 0
    text-align: right
    margin-right: 8px
    font-size: 12px
    opacity: 0.7
    cursor: default
  button.toggle
    font-size: 12px
    background: transparent
    padding-left: 0px
    margin: 0px
    margin-left: 12px
    border: none
    color: #3366ff
    &:active
      opacity: 0.8
  .frames
    padding-left: 12px
    user-select: auto
    -webkit-user-select: auto
    .frame-label
      font-size: 12px
      opacity: 0.7
      padding-top: 8px
</style>
