import { ApiSession } from '@/bindings/api/endpoints/session/ApiSession';
import { PostSession } from '@/bindings/api/endpoints/session/PostSession';
import { useAppContext } from '@/components/AppProvider';
import { SessionData } from '@/persistence/SessionData';
import { FetchRequestInit } from 'expo/fetch';
import { useEffect, useState } from 'react';

// const BASE_API_URL = 'https://192.168.1.130:3000/api/v0';
// const BASE_API_URL = 'https://localhost:3000/api/v0';
// const BASE_API_URL = 'http://172.28.234.97:3000/api/v0';
const BASE_API_URL = 'https://sensor-server.juancb.ftp.sh:3000/api/v0';

export interface ReturnedError<E> {
    status: number;
    errorBody?: E;
}

export enum Error {
    NetworkError,
    ReturnedError,
    JsonError,
    InvalidLocalSession,
}

const doFetch = fetch;

export interface ErrorState<E> {
    errorType: Error;
    error?: ReturnedError<E>;
}

function errorText<E>(err: ErrorState<E>, displayBody: boolean): string {
    switch (err.errorType) {
        case Error.InvalidLocalSession:
            return `Invalid session set on local device, try to login again`;
        case Error.JsonError:
            return `Error when reading server response, can't know if operation was successfull, please reload app`;
        case Error.NetworkError:
            return `Faced network related error when requesting server, try again`;
        case Error.ReturnedError:
            return displayBody
                ? `The following error occured: ${err.error?.errorBody}`
                : `The server returned an error, try again`;
    }
}

interface InternalFetchProps {
    endpoint_path: string;
    init: FetchRequestInit;
    sessionData: SessionData | undefined;
    setJwt: (jwt: string | undefined) => void;
}

type Rerun = boolean;

// Throws Error slice
async function _fetchApi<R>(props: InternalFetchProps): Promise<[R, boolean] | Rerun> {
    let { endpoint_path, init, sessionData } = props;

    let res;
    try {
        res = await doFetch(BASE_API_URL + endpoint_path, init);
    } catch (networkError) {
        console.log('networkError on first fetch: ', networkError);
        throw { errorType: Error.NetworkError } as ErrorState<undefined>;
    }

    console.debug('response: ', res);

    const readJson = res.headers.get('Content-type')?.includes('application/json');
    let response;
    if (readJson) {
        try {
            response = await res.json();
        } catch (jsonError) {
            console.error('JsonError: ', jsonError);
            throw { errorType: Error.JsonError } as ErrorState<undefined>;
        }
    } else {
        response = null;
    }

    if (!res.ok) {
        if (!sessionData) {
            let error: ErrorState<any> = {
                errorType: Error.InvalidLocalSession,
                error: { status: 401 },
            };

            throw error;
        }
        if (res.status === 401) {
            // Unauthorized
            console.warn('Unauthorized, calling renewJWT');
            try {
                let jwt = await renewJWT(sessionData);
                props.setJwt(jwt);
                return true;
            } catch (e: any) {
                if (typeof e === typeof Error.InvalidLocalSession) {
                    // i.e.: its an Error we threw
                    const error: ErrorState<undefined> = {
                        errorType: e,
                        error: { status: res.status, errorBody: response },
                    };
                    throw error;
                }
            }
        } else {
            let error: ErrorState<any> = {
                errorType: Error.ReturnedError,
                error: { status: res.status, errorBody: response },
            };
            throw error;
        }
    }

    return [response, res.status === 200];
}

async function renewJWT(session: SessionData): Promise<string> {
    console.log('renewJWT called with session: ', session);
    if (!session.username || !session.password) {
        throw Error.JsonError;
    }

    const body: PostSession = {
        'User': {
            username: session.username,
            raw_password: session.password,
        },
    };

    const init: FetchRequestInit = {
        body: JSON.stringify(body),
        method: 'POST',
        headers: [['Content-type', 'application/json']],
        credentials: 'include',
    };

    let res;
    try {
        res = await fetch(BASE_API_URL + '/session', init);
    } catch (networkError) {
        console.error('[renewJWT] networkError: ', networkError);
        throw Error.NetworkError;
    }
    console.log('[renewJWT] res: ', res);

    if (res.status !== 200) {
        if (res.status === 401 || res.status === 422) {
            console.debug('Invalid SessionData');
            await session.deleteSession();
        } else {
            console.error('Error received: ', res.status);
            throw Error.ReturnedError;
        }
    }

    try {
        let json: ApiSession;
        json = await res.json();
        console.log('GetSession response:', json);
        return json.access_token;
    } catch (e) {
        console.error('UNEXPECTED JSON ERROR FROM GET SESSION API', e);
        throw Error.JsonError;
    }
}

export default function useApi<B, R, E>(
    endpoint_path: string,
    method: 'GET' | 'POST' | 'PUT' | 'DELETE' | undefined,
    displayBody: boolean,
    body?: B,
    urlParams?: string[][],
) {
    const ctx = useAppContext();

    const [loading, setLoading] = useState<boolean>(false);
    const [error, setError] = useState<ErrorState<E> | undefined>(undefined);
    const [response, setResponse] = useState<R | undefined>(undefined);
    const [returnedOk, setReturnedOk] = useState<boolean | undefined>(undefined);

    useEffect(() => {
        if (!method) {
            return;
        }

        const controller = new AbortController();
        const signal = controller.signal;
        const stableBody = JSON.stringify(body);

        const fetchApi = async () => {
            setLoading(true);
            setError(undefined);
            setResponse(undefined);

            let init: FetchRequestInit = {
                body: stableBody,
                method: method,
                signal,
                credentials: 'include',
            };

            let headers: [string, string][] = [];

            if (method !== 'GET') {
                headers.push(['Content-type', 'application/json']);
            }
            if (ctx.jwt) {
                headers.push(['Authorization', 'Bearer ' + ctx.jwt]);
            }

            init.headers = headers;

            const parsed_params = urlParams
                ? '?' + new URLSearchParams(urlParams as string[][]).toString()
                : '';

            const props: InternalFetchProps = {
                endpoint_path: endpoint_path + parsed_params,
                init,
                sessionData: ctx.sessionData,
                setJwt: ctx.setJwt,
            };

            try {
                const ret = await _fetchApi<R>(props);
                if (typeof ret === 'boolean') {
                    throw 'Shortcuting api call due to JWT change';
                }
                const [r, ok] = ret;
                setError(undefined);
                setReturnedOk(ok);
                setResponse(r);
            } catch (e: any) {
                if (e.name !== 'AbortError') {
                    console.debug('setting api error to: ', e);
                    setError(e as ErrorState<E>);
                }
                setReturnedOk(false);
            } finally {
                if (!signal.aborted) {
                    setLoading(false);
                }
            }
        };

        fetchApi();

        return () => {
            controller.abort('useApi dependencies changed');
            setError(undefined);
            setLoading(false);
        };
    }, [urlParams, endpoint_path, method, body, ctx.sessionData, ctx.setJwt, ctx.jwt]);

    const formattedError = error ? errorText(error, displayBody) : null;

    const clearError = () => {
        setError(undefined);
    };

    return { response, loading, error, formattedError, clearError, returnedOk };
}
