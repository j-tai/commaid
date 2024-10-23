<script lang="ts">
    import { onMount } from 'svelte';

    let currentNotif: Notification | null = null;

    onMount(() => {
        Notification.requestPermission();
    });

    export function notify() {
        if (Notification.permission !== 'granted') return;
        if (document.hasFocus()) return;
        if (currentNotif) return;
        currentNotif = new Notification('ComMaid', { body: 'Someone is typing.' });
        currentNotif.addEventListener('close', () => {
            currentNotif = null;
        });
    }
</script>

<!-- Close notification once tab is refocused -->
<svelte:document
    onfocus={() => {
        if (document.hasFocus()) {
            currentNotif?.close();
            currentNotif = null;
        }
    }}
    onvisibilitychange={() => {
        if (!document.hidden) {
            currentNotif?.close();
            currentNotif = null;
        }
    }}
/>
