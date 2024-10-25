<script lang="ts">
    import type { Settings } from '$lib/settings.svelte';

    let { settings = $bindable() }: { settings: Settings } = $props();
</script>

<div class="container">
    <h2>Settings</h2>
    <label>
        <span>Font size</span>
        <input type="range" min={2} max={12} step={0.5} bind:value={settings.fontSize} />
        <span>{settings.fontSize}</span>
    </label>
    <label>
        <span>Font opacity</span>
        <input type="range" min={5} max={100} step={5} bind:value={settings.opacity} />
        <span>{settings.opacity}%</span>
    </label>
    <div class="card">
        <p style={settings.css()}>Text looks like this</p>
    </div>
</div>

<style>
    label {
        margin: 1em 0;
        width: 100%;
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 1em;
    }

    .card {
        font-size: 4svmin; /* for card padding */
        height: 13svmin;
        overflow: hidden;

        > p {
            text-wrap: nowrap;
        }
    }
</style>
