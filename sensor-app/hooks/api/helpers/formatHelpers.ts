export function isAlphanumeric(char: string): boolean {
    if (char.length !== 1) {
        return false;
    }
    return /^[a-zA-Z0-9]$/.test(char);
}

export function emailMatchesRegex(email: string): boolean {
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
}

export const hasControlChars = (str: string): boolean => {
    const controlCharRegex = /[\x00-\x1F\x7F]/;
    return controlCharRegex.test(str);
};
