<script lang="ts">
	import { onMount } from 'svelte';
	import { cn } from '$lib/utils';

	let {
		width = $bindable(),
		min,
		max,
		fallback,
		side,
		key
	}: {
		width: number;
		min: number;
		max: number;
		fallback: number;
		side: 'left' | 'right';
		key: string;
	} = $props();

	let drag: { x: number; w: number } | null = $state(null);

	const clamp = (n: number) => Math.min(max, Math.max(min, Math.round(n)));
	const sign = $derived(side === 'right' ? 1 : -1);

	function save() {
		try {
			localStorage.setItem(`cwdb:width:${key}`, String(width));
		} catch {}
	}

	onMount(() => {
		try {
			const stored = Number(localStorage.getItem(`cwdb:width:${key}`));
			if (stored) width = clamp(stored);
		} catch {}
	});

	function onkeydown(e: KeyboardEvent) {
		const dir = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0;
		if (!dir) return;
		e.preventDefault();
		width = clamp(width + dir * sign * 16);
		save();
	}
</script>

<div
	role="slider"
	aria-label="Resize sidebar"
	aria-valuenow={width}
	aria-valuemin={min}
	aria-valuemax={max}
	tabindex="0"
	class={cn(
		'absolute inset-y-0 z-10 hidden w-1.5 cursor-col-resize touch-none outline-none hover:bg-ring/40 focus-visible:bg-ring/60 lg:block',
		side === 'right' ? '-right-[3px]' : '-left-[3px]',
		drag && 'bg-ring/60'
	)}
	onpointerdown={(e) => {
		e.preventDefault();
		e.currentTarget.setPointerCapture(e.pointerId);
		drag = { x: e.clientX, w: width };
	}}
	onpointermove={(e) => drag && (width = clamp(drag.w + sign * (e.clientX - drag.x)))}
	onpointerup={() => {
		drag = null;
		save();
	}}
	ondblclick={() => {
		width = fallback;
		save();
	}}
	{onkeydown}
></div>
