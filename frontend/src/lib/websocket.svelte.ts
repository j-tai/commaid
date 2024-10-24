import { untrack } from 'svelte';

export class Socket {
    public state = $state(SocketState.Connecting);
    private ws!: WebSocket;
    private lastAttempt = 0;

    constructor(
        private url: string | URL,
        private onMessage: (text: string) => void,
    ) {
        this.reconnect();

        $effect(() => {
            // Reconnect on error
            if (this.state === SocketState.Error) {
                untrack(() => this.autoReconnect());
            }
        });
    }

    private autoReconnect() {
        if (Date.now() - this.lastAttempt > 5000) {
            this.reconnect();
        }
    }

    reconnect() {
        this.lastAttempt = Date.now();
        this.state = SocketState.Connecting;
        this.ws = new WebSocket(this.url);
        this.ws.addEventListener('open', () => (this.state = SocketState.Open));
        this.ws.addEventListener('close', () => (this.state = SocketState.Error));
        this.ws.addEventListener('error', () => (this.state = SocketState.Error));
        this.ws.addEventListener('message', (event) => this.onMessage(event.data));
    }

    send(value: string) {
        this.ws.send(value);
    }
}

export enum SocketState {
    Connecting = 'Connecting',
    Open = 'Open',
    Error = 'Error',
}
