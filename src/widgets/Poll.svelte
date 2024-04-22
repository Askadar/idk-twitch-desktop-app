<script>
	import Input from '../components/Input.svelte'

	let pollLive = false
	let pollTitle = 'A or B what you choose chat'

	let key1 = '1'
	let option1 = 'A is the correct choice'

	let key2 = ''
	let option2 = 'how could it not be B'
</script>

<div class="flex flex-col px-3 py-4 gap-3 w-112">
	<aside class="flex flex-row gap-2 align-base">
		<h2 class="text-xl/none">Quick poll</h2>
		<button on:click={() => (pollLive = !pollLive)}>toggle poll demo</button>
	</aside>
	{#if pollLive}
		<main class="flex flex-col gap-4 px-6 py-3 bg-white rounded-lg">
			<p class="text-base/none">Live results (this is a lie): {pollTitle}</p>

			<div class="flex flex-row gap-2">
				<span>#{key1}</span>
				<span>{option1}</span>
				<span class="grow-1"></span>
				<span>127 votes</span>
				<span>37%</span>
			</div>

			<div class="flex flex-row gap-2">
				<span>#{key2}</span>
				<span>{option2}</span>
				<span class="grow-1"></span>
				<span>216 votes</span>
				<span>63%</span>
			</div>
		</main>
	{:else}
		<main class="flex flex-col gap-4 px-6 py-3 bg-white rounded-lg">
			<p class="text-base/none">Start a new poll</p>

			<Input
				label="Polling for"
				placeholder="Provide a short description what you're polling for"
				bind:value={pollTitle}
			/>

			<div class="flex flex-row gap-1">
				<Input class="grow-0" label="key 1" placeholder="#1" bind:value={key1} />
				<Input label="option" placeholder="Option 1 label" bind:value={option1} />
			</div>

			<div class="flex flex-row gap-1">
				<Input class="grow-0" label="key 2" placeholder="#2" bind:value={key2} />
				<Input label="option" placeholder="Option 2 label" bind:value={option2} />
			</div>
		</main>
	{/if}
</div>
