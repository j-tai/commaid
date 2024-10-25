<script lang="ts">
    import { page } from '$app/stores';
    import QRCode from 'qrcode';

    let imageUrl = $state(new Promise<string>(() => {}));

    $effect(() => {
        imageUrl = QRCode.toDataURL($page.url.href, {
            scale: 1,
            margin: 2,
        });
    });

    let pageUrl = $derived.by(() => {
        const url = $page.url.href.replace(/^https?:\/\//, '');
        const slash = url.indexOf('/');
        if (slash < 0) {
            return [url, ''];
        } else {
            return [url.substring(0, slash), url.substring(slash)];
        }
    });
</script>

<div class="container">
    <h2>To see what I'm typing in real time, scan the QR code or enter the link.</h2>
    <div class="qr">
        {#await imageUrl}
            <p>Generating QR code...</p>
        {:then imageUrl}
            <img src={imageUrl} alt="QR code" />
        {/await}
    </div>
    <!-- Allow a line break before the first slash -->
    <p>
        <span>{pageUrl[0]}</span><wbr /><span>{pageUrl[1]}</span>
    </p>
</div>

<style lang="postcss">
    .qr {
        aspect-ratio: 1 / 1;
        width: 100%;
        max-width: 400px;
        margin: 0 auto;

        > * {
            width: 100%;
            height: 100%;
        }

        > img {
            image-rendering: pixelated;
        }
    }

    h2 {
        text-align: center;
    }

    p {
        text-align: center;
        margin-top: 0.3em;
        font-weight: 300;
        font-size: 250%;

        > span {
            white-space: nowrap;
        }
    }
</style>
