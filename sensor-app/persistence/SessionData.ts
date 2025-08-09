import * as SecureStore from 'expo-secure-store';

enum SessionKeys {
    USERNAME = 'USERNAME',
    HASHED_PASSWORD = 'HASHED_PASSWORD',
    API_ID = 'API_ID',
}

type key = string | null;

async function setSessionData(
    api_id: string,
    hashed_password: string,
    username: string,
): Promise<void> {
    const promises = [
        SecureStore.setItemAsync(SessionKeys.API_ID, api_id),
        SecureStore.setItemAsync(SessionKeys.HASHED_PASSWORD, hashed_password),
        SecureStore.setItemAsync(SessionKeys.USERNAME, username),
    ];

    await Promise.all(promises);
}

async function deleteSessionData(): Promise<void> {
    const promises = [
        SecureStore.deleteItemAsync(SessionKeys.USERNAME),
        SecureStore.deleteItemAsync(SessionKeys.API_ID),
        SecureStore.deleteItemAsync(SessionKeys.HASHED_PASSWORD),
    ];

    await Promise.all(promises);
}

async function loadSessionData(): Promise<[key, key, key]> {
    const promises = [
        SecureStore.getItemAsync(SessionKeys.USERNAME),
        SecureStore.getItemAsync(SessionKeys.API_ID),
        SecureStore.getItemAsync(SessionKeys.HASHED_PASSWORD),
    ];

    let [username, api_key, hashed_password]: [
        key | undefined,
        key | undefined,
        key | undefined,
    ] = [null, null, null];

    try {
        [username, api_key, hashed_password] = await Promise.all(promises);
    } catch (e) {
        console.error('Error thrown onSecureStore.getItemAsync: ', e);
        return [null, null, null];
    }

    if (
        username === undefined ||
        api_key === undefined ||
        hashed_password === undefined
    ) {
        console.log(
            `some is undefined... username: ${username}, api_id: ${api_key}, hashed_password: ${hashed_password}`,
        );
        throw 'Some was undefined';
    }

    return [username, api_key, hashed_password];
}

export class SessionData {
    private static _instance: SessionData | Promise<SessionData> | null = null;
    private static _initializing: Promise<SessionData> | null = null;

    private _api_id: string | null = null;
    private _hashed_password: string | null = null;
    private _username: string | null = null;

    private constructor(
        api_id: string | null,
        hashed_password: string | null,
        username: string | null,
    ) {
        this._api_id = api_id;
        this._hashed_password = hashed_password;
        this._username = username;
    }

    static async create(): Promise<SessionData> {
        if (this._instance) return this._instance;
        if (this._initializing) return this._initializing;

        this._initializing = (async () => {
            console.debug('[SessionData] initializing...');
            const [username, api_id, hashed_password] = await loadSessionData();
            this._instance = new SessionData(api_id, hashed_password, username);
            this._initializing = null;
            return this._instance;
        })();

        return this._initializing;
    }

    all_set(): boolean {
        return (
            this._api_id !== null &&
            this._hashed_password !== null &&
            this._username !== null
        );
    }

    async deleteSession() {
        // Can throw
        await deleteSessionData();
        this._api_id = null;
        this._hashed_password = null;
        this._username = null;
    }

    // Can throw
    async setSession(api_id: string, hashed_password: string, username: string) {
        await setSessionData(api_id, hashed_password, username);

        this._api_id = api_id;
        this._hashed_password = hashed_password;
        this._username = username;
    }

    get api_id(): string | null {
        return this._api_id;
    }

    get hashed_password(): string | null {
        return this._hashed_password;
    }

    get username(): string | null {
        return this._username;
    }
}
