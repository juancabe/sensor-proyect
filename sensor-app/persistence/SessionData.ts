import * as SecureStore from 'expo-secure-store';

enum SessionKeys {
    USERNAME = 'USERNAME',
    PASSWORD = 'PASSWORD',
}

type key = string | null;

async function setSessionData(username: string, password: string): Promise<void> {
    const promises = [
        SecureStore.setItemAsync(SessionKeys.USERNAME, username),
        SecureStore.setItemAsync(SessionKeys.PASSWORD, password),
    ];

    await Promise.all(promises);
}

async function deleteSessionData(): Promise<void> {
    const promises = [
        SecureStore.deleteItemAsync(SessionKeys.USERNAME),
        SecureStore.deleteItemAsync(SessionKeys.PASSWORD),
    ];

    await Promise.all(promises);
}

async function loadSessionData(): Promise<[key, key]> {
    const promises = [
        SecureStore.getItemAsync(SessionKeys.USERNAME),
        SecureStore.getItemAsync(SessionKeys.PASSWORD),
    ];

    let [username, password]: [key | undefined, key | undefined] = [null, null];

    try {
        [username, password] = await Promise.all(promises);
    } catch (e) {
        console.error('Error thrown onSecureStore.getItemAsync: ', e);
        return [null, null];
    }

    if (username === undefined || password === undefined) {
        console.log(`some is undefined... username: ${username}, password: ${password}`);
        throw 'Some was undefined';
    }

    return [username, password];
}

export class SessionData {
    private static _instance: SessionData | Promise<SessionData> | null = null;
    private static _initializing: Promise<SessionData> | null = null;

    private _password: string | null = null;
    private _username: string | null = null;

    private constructor(username: string | null, password: string | null) {
        this._password = password;
        this._username = username;
    }

    static async create(): Promise<SessionData> {
        if (this._instance) return this._instance;
        if (this._initializing) return this._initializing;

        this._initializing = (async () => {
            console.debug('[SessionData] initializing...');
            const [username, password] = await loadSessionData();
            this._instance = new SessionData(username, password);
            this._initializing = null;
            return this._instance;
        })();

        return this._initializing;
    }

    all_set(): boolean {
        return this._password !== null && this._username !== null;
    }

    async deleteSession() {
        // Can throw
        await deleteSessionData();
        this._password = null;
        this._username = null;
    }

    // Can throw
    async setSession(username: string, password: string) {
        await setSessionData(username, password);

        this._username = username;
        this._password = password;
    }

    get password(): string | null {
        return this._password;
    }

    get username(): string | null {
        return this._username;
    }
}
