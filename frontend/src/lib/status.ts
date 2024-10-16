export interface Status {
    clients: number | null;
    text: string | null;
}

export function parseStatus(status: string): Status {
    const dollarSign = status.indexOf('$');
    const [optionsText, text] =
        dollarSign === -1
            ? [status, null]
            : [status.substring(0, dollarSign), status.substring(dollarSign + 1)];

    const options = Object.fromEntries(
        optionsText
            .split(',')
            .filter((opt) => opt)
            .map((opt) => [opt.charAt(0), opt.substring(1)]),
    );

    return {
        clients: parseInt(options.n) || null,
        text,
    };
}
