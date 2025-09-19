export function safeGet<T extends object>(obj: T, key: string): T[keyof T] | undefined {
    return key in obj ? obj[key as keyof T] : undefined;
}

export function objectNumberKeysToArray(obj: any) {
    const numberKeys = Object.entries(obj)
        .filter(([_, v]) => typeof v === 'number')
        .map(([k]) => k);

    return numberKeys.map((key) => [key, safeGet(obj, key)]);
}
