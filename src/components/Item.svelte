<script lang="ts" context="module">
  export type Item = {
    path: string
    title: string
  }
  let x = 0
</script>

<script lang="ts">
  export let item: Item
</script>

<p>path: {item.path}</p>
<p>title: {item.title}</p>
