<script lang="ts">
    import { onMount } from 'svelte';

    let currentNotif: Notification | null = null;
    let nextNotifTime = 0;

    onMount(() => {
        Notification.requestPermission();
    });

    export function notify() {
        if (Notification.permission !== 'granted') return;
        if (document.hasFocus()) return;
        if (Date.now() < nextNotifTime) return;
        currentNotif = new Notification('ComMaid', { body: 'Someone is typing.' });
        currentNotif.addEventListener('close', () => (currentNotif = null));
        nextNotifTime = Date.now() + 10_000;
    }

    function reset() {
        currentNotif?.close();
        currentNotif = null;
        nextNotifTime = 0;
    }
</script>

<!-- Close notification once tab is refocused -->
<svelte:document
    onfocus={() => {
        if (document.hasFocus()) reset();
    }}
    onvisibilitychange={() => {
        if (!document.hidden) reset();
    }}
/>
