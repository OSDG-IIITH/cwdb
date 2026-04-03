<script lang="ts">
	let { title } = $props();
</script>

<header
	class="support-[backdrop-filter]:bg-background/60 sticky top-0 z-10 flex items-center justify-between border-b border-border/40 bg-background/95 px-6 py-4 backdrop-blur"
>
	<div class="flex items-center gap-2 font-mono text-xl font-bold tracking-tight">cwdb</div>
	<div class="text-md absolute left-1/2 -translate-x-1/2 font-semibold">
		{title}
	</div>
	<div class="w-[60px]"></div>
</header>
