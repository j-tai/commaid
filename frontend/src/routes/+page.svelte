<script lang="ts">
    import { page } from '$app/stores';
    import NotificationManager from '$lib/NotificationManager.svelte';
    import EditorPanel from '$lib/panel/EditorPanel.svelte';
    import { Panel } from '$lib/panel/panel';
    import PanelContainer from '$lib/panel/PanelContainer.svelte';
    import PanelTab from '$lib/panel/PanelTab.svelte';
    import SettingsPanel from '$lib/panel/SettingsPanel.svelte';
    import SharePanel from '$lib/panel/SharePanel.svelte';
    import WelcomePanel from '$lib/panel/WelcomePanel.svelte';
    import { generateRoom } from '$lib/room';
    import { Settings } from '$lib/settings.svelte';
    import { parseStatus } from '$lib/status';
    import { Socket, SocketState } from '$lib/websocket.svelte';
    import { onMount } from 'svelte';

    // Model state
    let clients = $state(1);
    let text = $state('');
    let settings = $state(new Settings());

    let roomName = $derived($page.url.hash.replace(/^#/, ''));
    let socket = $state<Socket | null>(null);
    let notifications = $state<NotificationManager>();

    // View state
    let panel = $state(Panel.Welcome);
    let defaultPanel = $derived(roomName ? Panel.Editor : Panel.Welcome);
    let statusBarClass = $derived('state-' + (socket?.state?.toLowerCase() ?? 'none'));

    onMount(() => {
        panel = defaultPanel;
    });

    // Opening the share panel will automatically enter a room
    $effect(() => {
        if (panel === Panel.Share && !roomName) {
            window.location.hash = generateRoom();
        }
    });

    // Connect when the room name changes
    $effect(() => connect());

    function openOffline() {
        window.location.hash = '';
        panel = Panel.Editor;
    }

    function openRandomRoom() {
        window.location.hash = generateRoom();
        panel = Panel.Editor;
    }

    function connect() {
        if (roomName) {
            socket = new Socket('/connect?' + new URLSearchParams({ room: roomName }), onReceive);
        } else {
            socket = null;
        }
    }

    function onReceive(data: string) {
        if (!data) return; // empty message is just a keepalive
        const status = parseStatus(data);
        if (status.text && status.text !== text) {
            // Send notification if text changed
            notifications?.notify();
        }
        clients = status.clients ?? clients;
        text = status.text ?? text;
    }

    function onEdit() {
        // If the socket is open, send the text whenever the user changes it
        // (but not when we merely receive a change from the server)
        if (socket?.state === SocketState.Open) {
            socket.send(text);
        }
    }
</script>

<svelte:head>
    <title>ComMaid</title>
</svelte:head>

{#if roomName}
    <NotificationManager bind:this={notifications} />
{/if}

<PanelContainer bind:panel back={defaultPanel} class={statusBarClass}>
    {#snippet status()}
        <p>
            {#if socket === null}
                You are not in a room. <button onclick={openRandomRoom}>Create one?</button>
            {:else if socket.state === SocketState.Connecting}
                Connecting...
            {:else if socket.state === SocketState.Open}
                Connected.
                {#if clients >= 3}
                    {clients - 1} other people in this room.
                {:else if clients === 2}
                    1 other person in this room.
                {:else}
                    <button onclick={() => (panel = Panel.Share)}>Invite people?</button>
                {/if}
            {:else if socket.state === SocketState.Error}
                Connection error. <button onclick={() => socket?.reconnect()}>Reconnect?</button>
            {/if}
        </p>
    {/snippet}

    {#snippet tabs()}
        <PanelTab bind:panel value={Panel.Welcome} icon="fi-ss-star" name="Home" />
        <PanelTab bind:panel value={Panel.Editor} icon="fi-ss-comment" name="Communicate" />
        <PanelTab bind:panel value={Panel.Share} icon="fi-ss-share" name="Share" />
        <PanelTab bind:panel value={Panel.Settings} icon="fi-ss-settings" name="Settings" />
    {/snippet}

    {#if panel === Panel.Welcome}
        <WelcomePanel onOpenOffline={openOffline} onOpenRoom={openRandomRoom} />
    {:else if panel === Panel.Editor}
        <EditorPanel bind:text {settings} socketState={socket?.state} {onEdit} />
    {:else if panel === Panel.Share}
        <SharePanel />
    {:else if panel === Panel.Settings}
        <SettingsPanel bind:settings />
    {/if}
</PanelContainer>

<style lang="postcss">
    p {
        text-align: center;
        font-weight: bold;

        button {
            text-decoration: underline;
        }
    }
</style>
