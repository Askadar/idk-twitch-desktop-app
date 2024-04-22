<script lang="ts">
	// import Greet from './lib/Greet.svelte'
	import Chat from './widgets/Chat.svelte'
	import Poll from './widgets/Poll.svelte'
	import Queue from './widgets/Queue.svelte'
	import Redeems from './widgets/Redeems.svelte'
</script>

<main class="app flex flex-row p-9 gap-8 h-full w-full m-0 bg-[#FFF9FC]">
	<aside class="w-20 h-full flex flex-col px-4 py-7 gap-2 bg-[#461D47] color-white rounded-xl">
		<i class="i-ph:house w-10 h-10"></i>
	</aside>
	<main class="flex-1 flex flex-wrap flex-row gap-6">
		<Poll />
		<Chat />
		<Queue />
		<Redeems />
	</main>
</main>
