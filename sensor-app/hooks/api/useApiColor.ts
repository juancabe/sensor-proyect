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
    const starting = initial
        ? API_COLORS.some((inColor) => inColor === initial)
            ? initial
            : API_COLORS[0]
        : API_COLORS[0];

    const [color, setColor] = useState<string>(starting);

    const fabricateError = (): string | undefined => {
        if (color.length !== API_COLORS[0].length) {
            return `Color length invalid`;
        }
        if (!API_COLORS.some((inColor) => inColor === color)) {
            return `The selected color (${color}) isn't available`;
        }
    };

    const error = fabricateError();
    const isValid = error === undefined;

    return { color, setColor, error, isValid, API_COLORS };
}
