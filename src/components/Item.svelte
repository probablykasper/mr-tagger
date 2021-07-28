<script context="module" lang="ts">
  export type Frame = {
    Text?: { id: string; value: string }
  }
  export type Item = {
    path: string
    frames: Frame[]
  }
  let x = 0 // to fix syntax highlighting
</script>

<script lang="ts">
  export let item: Item
  console.log(item)
</script>

<p>path: {item.path}</p>
{#each item.frames as frame}
  <div class="item">
    {#if frame.Text}
      <p>{frame.Text.id}: {frame.Text.value}</p>
    {/if}
  </div>
{/each}
