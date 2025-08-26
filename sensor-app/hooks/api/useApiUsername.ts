import { useState } from 'react';
import { isAlphanumeric } from './helpers/formatHelpers';

export function useApiUsername() {
    const MAX_LEN = 10;
    const MIN_LEN = 4;

    const [username, setUsername] = useState<string>('');

    const fabricateError = (): string | undefined => {
        if (username.length < MIN_LEN) {
            return `Username too short, at least ${MIN_LEN} characters`;
        }

        if (username.length > MAX_LEN) {
            return `Username too large, at most ${MAX_LEN} characters`;
        }

        for (const char of username) {
            if (!(isAlphanumeric(char) || char === '_')) {
                return `Characters like "${char}" are not allowed`;
            }
        }
    };

    const error = fabricateError();
    const isValid = error === undefined;

    return { username, setUsername, error, isValid };
}
