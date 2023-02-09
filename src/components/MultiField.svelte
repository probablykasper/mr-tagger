<script lang="ts">
	export let value: string[] = []
</script>

<div class="content">
	{#if value.length === 1}
		{value[0]}
	{:else}
		<!-- {#each ['Omri', 'Aero Chord'] as artist} -->
		{#each value as item}
			<span class="item">{item}</span>
		{/each}
	{/if}
</div>

<style lang="sass">
  .content
    font-size: 13px
    display: contents // safari selection fix
    user-select: auto
    -webkit-user-select: auto
    .item
      margin-right: 6px
      background-color: rgba(#ffffff, 0.05)
      border: 1px solid rgba(#ffffff, 0.1)
      padding: 2px 4px
      border-radius: 2px
</style>
