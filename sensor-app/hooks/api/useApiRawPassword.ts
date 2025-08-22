import { useState } from 'react';
import { hasControlChars } from './helpers/formatHelpers';

export function useApiRawPassword() {
    const MAX_LEN = 64;
    const MIN_LEN = 5;

    const [password, setPassword] = useState<string>('');

    const fabricateError = (): string | undefined => {
        if (password.length < MIN_LEN) {
            return `Password too short, at least ${MIN_LEN} characters`;
        }

        if (password.length > MAX_LEN) {
            return `Password too large, at most ${MAX_LEN} characters`;
        }

        if (!(password === password.trim())) {
            return 'Password cannot start or end with whitespaces';
        }

        if (hasControlChars(password)) {
            return 'Password cannot include control characters';
        }
    };

    const error = fabricateError();
    const isValid = error === undefined;

    return { password, setPassword, error, isValid };
}
