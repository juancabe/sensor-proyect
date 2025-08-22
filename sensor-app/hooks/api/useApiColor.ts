import { useState } from 'react';

export const API_COLORS = [
    '#FF0000',
    '#0000FF',
    '#FFFF00',
    '#008000',
    '#FFA500',
    '#800080',
    '#FFFFFF',
    '#000000',
    '#808080',
];

export function useApiColor(initial?: string) {
    const LEN = 10;
    const starting = initial
        ? initial in API_COLORS
            ? initial
            : API_COLORS[0]
        : API_COLORS[0];

    const [color, setColor] = useState<string>(starting);

    const fabricateError = (): string | undefined => {
        if (color.length !== LEN) {
            return `Color length invalid`;
        }

        if (!(color in API_COLORS)) {
            return `The selected color (${color}) isn't available`;
        }
    };

    const error = fabricateError();
    const isValid = error === undefined;

    return { color, setColor, error, isValid, API_COLORS };
}
