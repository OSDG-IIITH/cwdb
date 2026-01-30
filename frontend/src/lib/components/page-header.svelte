<script lang="ts">
    let { title } = $props();
</script>

<header class="border-b border-border/40 px-6 py-4 flex items-center justify-between sticky top-0 bg-background/95 backdrop-blur z-10 support-[backdrop-filter]:bg-background/60">
    <div class="flex items-center gap-2 font-bold text-xl tracking-tight font-mono">
        cwdb
    </div>
    <div class="absolute left-1/2 -translate-x-1/2 font-semibold text-md">
        {title}
    </div>
    <div class="w-[60px]"></div>
</header>
