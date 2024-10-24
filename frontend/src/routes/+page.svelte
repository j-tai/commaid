<script lang="ts">
    import { page } from '$app/stores';
    import NotificationManager from '$lib/NotificationManager.svelte';
    import EditorPanel from '$lib/panel/EditorPanel.svelte';
    import { Panel } from '$lib/panel/panel';
    import PanelContainer from '$lib/panel/PanelContainer.svelte';
    import PanelTab from '$lib/panel/PanelTab.svelte';
    import SettingsPanel from '$lib/panel/SettingsPanel.svelte';
    import WelcomePanel from '$lib/panel/WelcomePanel.svelte';
    import { generateRoom } from '$lib/room';
    import { Settings } from '$lib/settings.svelte';
    import { parseStatus } from '$lib/status';
    import { Socket, SocketState } from '$lib/websocket.svelte';
    import { onMount } from 'svelte';

    let panel = $state<Panel>(Panel.Welcome);

    let clients = $state(1);
    let text = $state('');
    let settings = $state(new Settings());

    let randomRoomName = $state('');
    onMount(() => (randomRoomName = generateRoom()));

    let roomName = $derived($page.url.hash.replace(/^#/, ''));
    let socket = $state<Socket | null>(null);
    let notifications = $state<NotificationManager>();

    // Derived things
    let statusBarClass = $derived('state-' + (socket?.state?.toLowerCase() ?? 'none'));

    function connect() {
        if (roomName) {
            socket = new Socket('/connect?' + new URLSearchParams({ room: roomName }), onReceive);
            panel = Panel.Editor;
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

    $effect(() => {
        // Connect when the room name changes
        connect();
    });
</script>

<svelte:head>
    <title>ComMaid</title>
</svelte:head>

{#if roomName}
    <NotificationManager bind:this={notifications} />
{/if}

<PanelContainer bind:panel back={Panel.Editor} class={statusBarClass}>
    {#snippet status()}
        <p>
            {#if panel === Panel.Settings}
                Settings
            {:else if socket === null}
                You are not in a room. <a href="#{randomRoomName}">Create one?</a>
            {:else if socket.state === SocketState.Connecting}
                Connecting...
            {:else if socket.state === SocketState.Open}
                Connected.
                {#if clients >= 3}
                    {clients - 1} other people in this room.
                {:else if clients === 2}
                    1 other person in this room.
                {:else}
                    Share the URL to invite others to the room.
                {/if}
            {:else if socket.state === SocketState.Error}
                Connection error. <button onclick={() => socket?.reconnect()}>Reconnect?</button>
            {/if}
        </p>
    {/snippet}

    {#snippet tabs()}
        <PanelTab bind:panel value={Panel.Welcome} icon="fi-ss-star" name="Home" />
        <PanelTab bind:panel value={Panel.Editor} icon="fi-ss-comment" name="Communicate" />
        <PanelTab bind:panel value={Panel.Settings} icon="fi-ss-settings" name="Settings" />
    {/snippet}

    {#if panel === Panel.Welcome}
        <WelcomePanel />
    {:else if panel === Panel.Editor}
        <EditorPanel bind:text {settings} socketState={socket?.state} {onEdit} />
    {:else if panel === Panel.Settings}
        <SettingsPanel bind:settings />
    {/if}
</PanelContainer>

<style lang="postcss">
    p {
        text-align: center;
        font-weight: bold;

        a,
        button {
            text-decoration: underline;
        }
    }
</style>
