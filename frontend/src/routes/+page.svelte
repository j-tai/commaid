<script lang="ts">
    import { page } from '$app/stores';
    import { generateRoom } from '$lib/room';
    import { parseStatus } from '$lib/status';
    import { onMount } from 'svelte';
    import { Socket, SocketState } from '../lib/websocket.svelte';

    let clients = $state(1);
    let text = $state('');

    let randomRoomName = $state('');
    onMount(() => (randomRoomName = generateRoom()));

    let roomName = $derived($page.url.hash.replace(/^#/, ''));
    let socket = $state<Socket | null>(null);
    let disabled = $derived(socket?.state === SocketState.Connecting);

    function connect() {
        if (roomName) {
            socket = new Socket('/connect?' + new URLSearchParams({ room: roomName }), onReceive);
        } else {
            socket = null;
        }
    }

    function onReceive(data: string) {
        const status = parseStatus(data);
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

<div class="container">
    <p
        class:state-none={socket === null}
        class:state-connecting={socket?.state === SocketState.Connecting}
        class:state-open={socket?.state === SocketState.Open}
        class:state-error={socket?.state === SocketState.Error}
    >
        {#if socket === null}
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
    <!-- svelte-ignore a11y_autofocus -->
    <textarea
        spellcheck="false"
        placeholder={disabled ? '' : 'Start typing here...'}
        autofocus
        bind:value={text}
        {disabled}
        oninput={onEdit}
    ></textarea>
</div>
