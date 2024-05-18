<script>
	import { emit, listen } from '@tauri-apps/api/event'
	import { onDestroy } from 'svelte'

	// listen to the `click` event and get a function to remove the event listener
	// there's also a `once` function that subscribes to an event and automatically unsubscribes the listener on the first event
	/**
	 *
	 * @typedef {{ name: string; message: string; emotes: {char_range: {start: number, end: number}, code: string, id: string}[] }} Message
	 * @type {Message[]}
	 */
	let messages = []
	let unsub;
	listen('new-message', (event) => {
		console.log(event.payload)
		/**  @type {Message} */
		const m = event.payload
		for (let e of m.emotes) {
			m.message = m.message.replaceAll(e.code, `<img class="h-8 inline-block" src="https://static-cdn.jtvnw.net/emoticons/v2/${e.id}/default/light/2.0" />`)
		}
		messages.push(m)
		messages = messages.slice(-10)
		// event.event is the event name (useful if you want to use a single callback fn for multiple event types)
		// event.payload is the payload object
	}).then((u) => (unsub = u))
	onDestroy(() => unsub?.())
</script>

<div class="flex flex-col px-3 py-4 gap-3">
	<aside class="flex flex-row gap-2 align-base">
		<h2 class="text-xl/none">Chat</h2>
	</aside>
	<div class="px-6 py-3 bg-white rounded-lg gap-1">
		{#each messages as message}
			<p><span class="mr-1">{message.name}:</span><span class="inline-block">{@html message.message}</span></p>
		{/each}
	</div>
</div>
