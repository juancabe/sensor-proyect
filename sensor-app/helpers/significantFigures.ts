export function toSignificantFigures(value: number, sf: number): string {
    if (!Number.isFinite(value)) return 'NaN'; // NaN, Infinity passthrough [web:31]
    if (sf < 1 || sf > 100) throw new RangeError('sf must be between 1 and 100'); // MDN range [web:37]
    if (Number.isInteger(value)) return `${value}`;
    return value.toPrecision(sf);
}
