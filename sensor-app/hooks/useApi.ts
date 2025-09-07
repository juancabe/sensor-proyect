import { ApiSession } from '@/bindings/api/endpoints/session/ApiSession';
import { PostSession } from '@/bindings/api/endpoints/session/PostSession';
import { useAppContext } from '@/components/AppProvider';
import { SessionData } from '@/persistence/SessionData';
import { FetchRequestInit } from 'expo/fetch';
import { useEffect, useState } from 'react';

const BASE_API_URL = 'http://192.168.1.130:3000/api/v0';

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

export type ErrorState<E> = [Error, ReturnedError<E>];

function errorText<E>(err: ErrorState<E>, displayBody: boolean): string {
    switch (err[0]) {
        case Error.InvalidLocalSession:
            return `Invalid session set on local device, try to login again`;
        case Error.JsonError:
            return `Error when reading server response, can't know if operation was successfull, please reload app`;
        case Error.NetworkError:
            return `Faced network related error when requesting server, try again`;
        case Error.ReturnedError:
            return displayBody
                ? `The following error occured: ${err[1]?.errorBody}`
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
        res = await fetch(BASE_API_URL + endpoint_path, init);
    } catch (networkError) {
        console.error('networkError: ', networkError);
        throw [Error.NetworkError];
    }

    console.debug('response: ', res);

    const readJson = res.headers.get('Content-type')?.includes('application/json');
    let response;
    if (readJson) {
        try {
            response = await res.json();
        } catch (jsonError) {
            console.error('JsonError: ', jsonError);
            throw [Error.JsonError];
        }
    } else {
        response = null;
    }

    if (!res.ok) {
        if (!sessionData) {
            let error: ErrorState<any> = [Error.InvalidLocalSession, { status: 401 }];

            throw error;
        }
        if (res.status === 401) {
            // Unauthorized
            let jwt = await renewJWT(sessionData);
            props.setJwt(jwt);
            return true;
        } else {
            let error: ErrorState<any> = [
                Error.ReturnedError,
                { status: res.status, errorBody: response },
            ];

            throw error;
        }
    }

    return [response, res.status === 200];
}

async function renewJWT(session: SessionData): Promise<string> {
    if (!session.username || !session.password) {
        throw [Error.JsonError];
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
    };

    let res;
    try {
        res = await fetch(BASE_API_URL + '/session', init);
    } catch (networkError) {
        console.error('[renewJWT] networkError: ', networkError);
        throw [Error.NetworkError];
    }
    console.log('[renewJWT] res: ', res);

    if (res.status !== 200) {
        if (res.status === 401 || res.status === 422) {
            console.debug('Invalid SessionData');
            await session.deleteSession();
        } else {
            console.error('Error received: ', res.status);
        }
    }

    try {
        let json: ApiSession;
        json = await res.json();
        console.log('GetSession response:', json);
        return json.access_token;
    } catch (e) {
        console.error('UNEXPECTED JSON ERROR FROM GET SESSION API', e);
        throw [Error.JsonError];
    }
}

export default function useApi<B, R, E>(
    endpoint_path: string,
    body: B,
    method: 'GET' | 'POST' | 'PUT' | 'DELETE' | undefined,
    displayBody: boolean,
) {
    const ctx = useAppContext();

    const [loading, setLoading] = useState<boolean>(false);
    const [error, setError] = useState<ErrorState<E> | undefined>(undefined);
    const [response, setResponse] = useState<R | undefined>(undefined);
    const [returnedOk, setReturnedOk] = useState<boolean | undefined>(undefined);

    const stableBody = JSON.stringify(body);

    useEffect(() => {
        if (!method) {
            return;
        }

        const controller = new AbortController();
        const signal = controller.signal;

        const fetchApi = async () => {
            setLoading(true);
            setError(undefined);
            setResponse(undefined);

            let init: FetchRequestInit = {
                body: stableBody,
                method: method,
                signal,
            };

            let headers: [string, string][] = [];

            if (method !== 'GET') {
                headers.push(['Content-type', 'application/json']);
            }
            if (ctx.jwt) {
                headers.push(['Authorization', 'Bearer ' + ctx.jwt]);
            }

            init.headers = headers;

            const props: InternalFetchProps = {
                endpoint_path,
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
            controller.abort();
        };
    }, [endpoint_path, method, stableBody, ctx.sessionData, ctx.setJwt, ctx.jwt]);

    const formattedError = error ? errorText(error, displayBody) : null;

    const clearError = () => {
        setError(undefined);
    };

    return { response, loading, error, formattedError, clearError, returnedOk };
}
