export function timeDisplay(date: Date): string {
    const diff = new Date().getTime() - date.getTime();
    const MILLIS = 1000;
    const SECONDS = 60;
    const MINUTES_30 = MILLIS * SECONDS * 30;
    const MINUTES_2 = MILLIS * SECONDS * 2;

    if (Math.abs(diff) > MINUTES_30) {
        return date.toUTCString();
    } else {
        if (diff > MINUTES_2) {
            return `${~~(diff / (MILLIS * SECONDS))} mins`;
        } else {
            return `${~~(diff / MILLIS)} secs`;
        }
    }
}
