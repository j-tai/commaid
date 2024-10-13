<script lang="ts">
    import { page } from '$app/stores';
    import { untrack } from 'svelte';
    import { Socket, SocketState } from '../lib/websocket.svelte';

    let text = $state('');

    let roomName = $derived($page.url.hash.replace(/^#/, ''));
    let socket = $state<Socket | null>(null);

    function connect() {
        if (roomName) {
            socket = new Socket(
                '/connect?' + new URLSearchParams({ room: roomName }),
                (value) => (text = value),
            );
        } else {
            socket = null;
        }
    }

    $effect(() => {
        // Connect when the room name changes
        connect();

        // If the socket is open, send the text whenever it changes
        $effect(() => {
            if (socket?.state === SocketState.Open) {
                socket.send(text);
            }
        });
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
            You are not in a room.
        {:else if socket.state === SocketState.Connecting}
            Connecting...
        {:else if socket.state === SocketState.Open}
            Connected.
        {:else if socket.state === SocketState.Error}
            Connection error.
        {/if}
    </p>
    <!-- svelte-ignore a11y_autofocus -->
    <textarea
        spellcheck="false"
        placeholder="Start typing here..."
        autofocus
        bind:value={text}
        disabled={socket?.state === SocketState.Connecting}
    ></textarea>
</div>
