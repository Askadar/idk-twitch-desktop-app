<script>
	export let value = ''
	export let label = '_missing label'
	export let placeholder = ''

	export const id = Math.random().toFixed(8)
	let className = ''
	export { className as class}
</script>

<label for={id} class={`flex flex-col relative mt-1 flex-1 min-w-14 ${className}`}>
	<span class="absolute left-4 -top-0.5em text-sm/none bg-white px-1 -mx-2">{label}</span>
	<input
		class="text-base/none px-3 py-1 border border-slate-600 rounded-md"
		{id}
		{placeholder}
		type="text"
		bind:value
	/>
</label>
