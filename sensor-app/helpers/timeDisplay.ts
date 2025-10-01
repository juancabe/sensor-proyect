export function timeDisplay(
    date: Date,
    displayIsAgo?: boolean,
    neverUTCString?: boolean,
): string {
    const diff = Date.now() - date.getTime(); // en milisegundos
    const isAgo = displayIsAgo ? ' ago' : '';

    const MS_PER_SECOND = 1000;
    const MS_PER_MINUTE = MS_PER_SECOND * 60;
    const THIRTY_MINUTES = MS_PER_MINUTE * 30;
    const TWO_MINUTES = MS_PER_MINUTE * 2;

    if (Math.abs(diff) > THIRTY_MINUTES && !neverUTCString) {
        return date.toUTCString();
    }

    if (diff > TWO_MINUTES || (neverUTCString && diff > TWO_MINUTES)) {
        return `${~~(diff / MS_PER_MINUTE)} mins` + isAgo;
    }

    return `${~~(diff / MS_PER_SECOND)} secs` + isAgo;
}
