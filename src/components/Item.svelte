<script context="module" lang="ts">
  export type Frame = {
    Text?: { id: string; value: string }
  }
  export type Image = {
    index: number
    total_images: number
    data: Uint8Array
    mime_type: string
  }
  export type Item = {
    path: string
    frames: Frame[]
  }
  let x = 0 // to fix syntax highlighting
</script>

<script lang="ts">
  import { runCmd } from '../scripts/helpers'

  export let item: Item
  console.log(item)

  let image: Image | null = null
  $: if (item) {
    image = null
    getImage(null)
  }
  async function getImage(index: number | null) {
    image = (await runCmd('get_image', { index })) as Image | null
  }
  async function removeArtwork() {
    if (image) {
      await runCmd('remove_image', { index: image.index })
      getImage(null)
    }
  }
</script>

<main>
  <div class="left">
    {#if image}
      <div class="cover">
        <img src={'data:' + image.mime_type + ';base64,' + image.data} alt="" />
      </div>
    {:else}
      <div class="svg-cover">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          preserveAspectRatio="xMidYMin meet"
          width="24"
          height="24"
          viewBox="0 0 24 24">
          <path
            d="M23 0l-15.996 3.585v13.04c-2.979-.589-6.004 1.671-6.004 4.154 0 2.137 1.671 3.221 3.485 3.221 2.155 0 4.512-1.528 4.515-4.638v-10.9l12-2.459v8.624c-2.975-.587-6 1.664-6 4.141 0 2.143 1.715 3.232 3.521 3.232 2.14 0 4.476-1.526 4.479-4.636v-17.364z" />
        </svg>
      </div>
    {/if}
    {#if image}
      <div>{image.index + 1} of {image.total_images}, {image.mime_type}</div>
      <div><button on:click={removeArtwork}>Remove</button></div>
    {/if}
  </div>
  <div class="right">
    <div>{item.path}</div>
    {#each item.frames as frame}
      <div class="item">
        {#if frame.Text}
          <p>{frame.Text.id}: {frame.Text.value}</p>
        {/if}
      </div>
    {/each}
  </div>
</main>

<style lang="sass">
  main
    display: flex
    margin: 12px
    font-size: 14px
  .left
    margin-right: 12px
    min-width: 150px
    width: 20%
  img
    width: 100%
  .svg-cover
    width: 100%
    padding-bottom: 100%
  svg
    width: 100%
    height: 100%
    padding: 28%
    box-sizing: border-box
    background-color: #2b2c31
    fill: #45464a
</style>
