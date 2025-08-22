import { useState } from 'react';
import { emailMatchesRegex } from './helpers/formatHelpers';

export function useApiEmail() {
    const MAX_LEN = 320;
    const MIN_LEN = 3;

    const [email, setEmail] = useState<string>('');

    const fabricateError = (): string | undefined => {
        if (email === null) {
            return;
        }

        if (email.length < MIN_LEN) {
            return `Email too short, at least ${MIN_LEN} characters`;
        }

        if (email.length > MAX_LEN) {
            return `Email too large, at most ${MAX_LEN} characters`;
        }

        if (!emailMatchesRegex(email)) {
            return `Invalid email, for example email@email.com is correct`;
        }
    };

    const error = fabricateError();
    const isValid = error === undefined;

    return { email, setEmail, error, isValid };
}
