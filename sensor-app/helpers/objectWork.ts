export function safeGet<T extends object>(obj: T, key: string): T[keyof T] | undefined {
    return key in obj ? obj[key as keyof T] : undefined;
}
