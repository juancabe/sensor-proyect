export function rgbWithAlpha(rgb: string, a: number) {
    const matched = rgb.match(/\d+/g);
    if (!matched) return;
    if (a < 0 || a > 1) return;
    const [r, g, b] = matched.map(Number);
    return `rgba(${r}, ${g}, ${b}, ${a})`;
}

export function hex6WithAlpha(hex: string, a: number) {
    if (a < 0 || a > 1) return;
    const padded = ~~(a * 0xff).toString(16).padStart(2, '0');
    return `${hex}${padded}`;
}
