<script lang="ts">
  import { invoke } from '@tauri-apps/api'
  import { popup } from './scripts/helpers'
  import ItemView from './components/Item.svelte'
  import type { Item } from './components/Item.svelte'
  let item: Item | null = null
  async function open() {
    item = (await invoke('open_dialog').catch(popup)) as any
  }
</script>

<main>
  {#if item}
    <ItemView {item} />
  {:else}
    <h1>Mr Tagger</h1>
    <button on:click={open}>Open</button>
  {/if}
</main>

<style lang="sass">
  :global(body)
    margin: 0
    font-family: Arial, Helvetica, sans-serif
    font-size: 18px
    background-color: #191B20
  main
    display: flex
    flex-direction: column
    justify-content: center
    align-items: center
    color: #e6e6e6
    min-height: 100vh
</style>
