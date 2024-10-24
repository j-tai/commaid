<script lang="ts">
    import type { Panel } from './panel';

    let {
        panel = $bindable(),
        value,
        name,
        icon,
    }: {
        panel: Panel;
        value: Panel;
        name: string;
        icon: string;
    } = $props();
</script>

<button class:sel={panel === value} onclick={() => (panel = value)} aria-label={name}>
    <i class="fi {icon}"></i>
</button>

<style lang="postcss">
    button {
        flex: none;
        padding: 0.5em;
        text-decoration: none;
    }

    .sel {
        background: var(--bg);
    }
</style>
