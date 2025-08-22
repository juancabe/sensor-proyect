import { ApiSession } from '@/bindings/api/endpoints/session/ApiSession';
import { PostSession } from '@/bindings/api/endpoints/session/PostSession';
import { useAppContext } from '@/components/AppProvider';
import { SessionData } from '@/persistence/SessionData';
import { FetchRequestInit } from 'expo/fetch';
import { useEffect, useState } from 'react';

const BASE_API_URL = 'localhost:3000/api/v0';

export interface ReturnedError<E> {
    status: number;
    errorBody: E;
}

export enum Error {
    NetworkError,
    ReturnedError,
    JsonError,
    InvalidLocalSession,
}

export type ErrorState<E> = [Error, ReturnedError<E>?];

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
}

// Throws Error slice
async function _fetchApi<R>(props: InternalFetchProps): Promise<R> {
    let { endpoint_path, init, sessionData } = props;

    let res;
    try {
        res = await fetch(BASE_API_URL + endpoint_path, init);
    } catch (networkError) {
        console.error('networkError: ', networkError);
        throw [Error.NetworkError];
    }

    const readJson = res.headers.get('content-type')?.includes('application/json');
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
            throw [
                Error.ReturnedError,
                response ? { status: res.status, errorBody: response } : undefined,
            ];
        } else {
            // TODO: Renew JWT
            await renewJWT(sessionData);
            return await _fetchApi(props);
        }
    }

    return response;
}

async function renewJWT(session: SessionData): Promise<void> {
    if (!session.username || !session.hashed_password) {
        throw [Error.JsonError];
    }

    const body: PostSession = {
        username: session.username,
        raw_password: session.hashed_password,
    };

    const init: FetchRequestInit = {
        body: JSON.stringify(body),
        method: 'POST',
    };

    let res;
    try {
        res = await fetch(BASE_API_URL + '/session', init);
    } catch (networkError) {
        console.error('[renewJWT] networkError: ', networkError);
        throw [Error.NetworkError];
    }
    // TODO: Check that JWT is set as cookie

    try {
        let json: ApiSession;
        json = await res.json();
        console.log('GetSession response:', json);
    } catch (e) {
        console.error('UNEXPECTED JSON ERROR FROM GET SESSION API', e);
        throw [Error.JsonError];
    }
}

export default function useApi<B, R, E>(
    endpoint_path: string,
    body: B,
    method: 'GET' | 'POST' | 'PUT' | 'DELETE' | undefined,
) {
    const ctx = useAppContext();

    const [loading, setLoading] = useState<boolean>(false);
    const [error, setError] = useState<ErrorState<E> | undefined>(undefined);
    const [response, setResponse] = useState<R | undefined>(undefined);

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

            const init: FetchRequestInit = {
                body: stableBody,
                method: method,
                signal,
            };

            const props: InternalFetchProps = {
                endpoint_path,
                init,
                sessionData: ctx.sessionData,
            };

            try {
                const r = await _fetchApi<R>(props);
                setResponse(r);
            } catch (e: any) {
                if (e.name !== 'AbortError') {
                    setError(e as ErrorState<E>);
                }
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
    }, [endpoint_path, method, stableBody, ctx.sessionData]);

    const formattedError = error ? errorText(error, body ? true : false) : null;

    return { response, loading, error, formattedError };
}
