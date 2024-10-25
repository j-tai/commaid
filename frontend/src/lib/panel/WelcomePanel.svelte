<script lang="ts">
    let {
        onOpenOffline,
        onOpenRoom,
    }: {
        onOpenOffline: () => void;
        onOpenRoom: () => void;
    } = $props();
</script>

<div class="container">
    <h2>Welcome to ComMaid</h2>

    <p>
        <b>ComMaid</b> is a communication aid for people who are unable or unwilling to speak. It is
        useful for both Deaf and non-Deaf people, such as people with situational mutism, or anyone who
        prefers typing over talking.
    </p>
    <p>
        You can use ComMaid offline without creating a room. If you create a room, you can share the
        link to allow others to see what you are typing in real time.
    </p>
    <p>
        The name "ComMaid" sounds similar to "comm aid", i.e., communication aid. You can also think
        of it as a "maid" who helps you communicate.
    </p>

    <button class="card hover" onclick={onOpenOffline}>
        <span>Use ComMaid offline</span>
        <i class="fi fi-ss-angle-right"></i>
    </button>
    <button class="card hover" onclick={onOpenRoom}>
        <span>Create a room</span>
        <i class="fi fi-ss-angle-right"></i>
    </button>
</div>

<style lang="postcss">
    p {
        margin: 0.5em 0;
    }

    button {
        margin-top: 0.5em;
        width: 100%;
        font-size: 200%;
        font-weight: 300;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    b {
        font-weight: bold;
    }
</style>
