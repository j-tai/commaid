export class Settings {
    fontSize = $state(8);
    opacity = $state(100);

    css(): string {
        return [`font-size: ${this.fontSize}svmin`, `opacity: ${this.opacity / 100}`].join(';');
    }
}
