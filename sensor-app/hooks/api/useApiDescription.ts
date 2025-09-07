import { useState } from 'react';
import { isAlphanumeric } from './helpers/formatHelpers';

export function useApiDescription() {
    const MAX_LEN = 50;
    const MIN_LEN = 5;

    const [description, setDescription] = useState<string | null>(null);

    const fabricateError = (): string | undefined => {
        if (description === null) {
            return undefined;
        }

        if (description.length < MIN_LEN) {
            return `Description too short, at least ${MIN_LEN} characters`;
        }

        if (description.length > MAX_LEN) {
            return `Description too large, at most ${MAX_LEN} characters`;
        }

        for (const char of description) {
            if (!(isAlphanumeric(char) || char === ' ')) {
                return `Characters like "${char}" are not allowed`;
            }
        }

        return undefined;
    };

    const error = fabricateError();
    const isValid = error === undefined;

    const publicSetDescription = (description: string | null) => {
        if (!description) {
            setDescription(null);
            return;
        }

        if (description.length < 1) {
            setDescription(null);
        } else {
            setDescription(description);
        }
    };

    return { description, setDescription: publicSetDescription, error, isValid };
}
