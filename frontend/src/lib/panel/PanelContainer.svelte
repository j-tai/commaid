<script lang="ts">
    import type { Snippet } from 'svelte';
    import { Panel } from './panel';
    import PanelTab from './PanelTab.svelte';

    let {
        class: classes = '',
        back,
        panel = $bindable(),
        children,
        status,
        tabs,
    }: {
        class?: string;
        back?: Panel | null | undefined;
        panel: Panel;
        children: Snippet;
        status: Snippet;
        tabs: Snippet;
    } = $props();
</script>

<div class="root {classes}">
    <div class="bar">
        {#if back && back !== panel}
            <div class="tabs">
                <PanelTab bind:panel value={back} name="Back" icon="fi-ss-angle-left" />
            </div>
        {/if}
        <div class="stretch">
            {@render status()}
        </div>
        <div class="tabs">
            {@render tabs()}
        </div>
    </div>
    <div class="stretch">
        {@render children()}
    </div>
</div>

<style lang="postcss">
    div {
        flex: none;
    }

    .stretch {
        flex: 1 1;
    }

    .root {
        height: 100%;
        width: 100%;
        display: flex;
        flex-direction: column;

        > .bar {
            display: flex;
            flex-direction: row;
            align-items: center;

            > .tabs {
                display: flex;
                flex-direction: row;
            }

            > .stretch {
                padding: 0.3em;
            }
        }
    }
</style>
