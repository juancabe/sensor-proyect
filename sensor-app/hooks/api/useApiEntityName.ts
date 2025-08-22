import { useState } from 'react';
import { isAlphanumeric } from './helpers/formatHelpers';

export function useApiEntityName() {
    const MAX_LEN = 15;
    const MIN_LEN = 3;

    const [name, setName] = useState<string>('');

    const fabricateError = (): string | undefined => {
        if (name.length < MIN_LEN) {
            return `Name too short, at least ${MIN_LEN} characters`;
        }

        if (name.length > MAX_LEN) {
            return `Name too large, at most ${MAX_LEN} characters`;
        }

        for (const char of name) {
            if (!isAlphanumeric(char) || !(char === ' ')) {
                return `Characters like "${char}" are not allowed`;
            }
        }
    };

    const error = fabricateError();
    const isValid = error === undefined;

    return { name, setName, error, isValid };
}
