<script lang="ts">
    import type { Settings } from '$lib/settings.svelte';
    import { SocketState } from '$lib/websocket.svelte';

    let {
        text = $bindable(),
        settings,
        socketState,
        onEdit,
    }: {
        text: string;
        settings: Settings;
        socketState?: SocketState | null | undefined;
        onEdit: (text: string) => void;
    } = $props();

    let disabled = $derived(socketState === SocketState.Connecting);
</script>

<!-- svelte-ignore a11y_autofocus -->
<textarea
    spellcheck="false"
    placeholder={disabled ? '' : 'Start typing here...'}
    autofocus
    bind:value={text}
    {disabled}
    oninput={(e) => onEdit(e.currentTarget.value)}
    style={settings.css()}
></textarea>

<style lang="postcss">
    textarea {
        width: 100%;
        height: 100%;
        resize: none;
        padding: 0.3em;
        background: unset;
        color: unset;
    }
</style>
