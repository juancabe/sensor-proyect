import { fetch, FetchRequestInit } from 'expo/fetch';

// export const BASE = 'http://sensor-server.juancb.ftp.sh:3000';
export const BASE = 'http://192.168.1.134:3000';

export class UnexpectedBodyError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'UnexpectedBodyError';
    }
}

export class FetchError extends Error {
    constructor(error: Error) {
        super(error.message);
        this.name = 'FetchError';
    }
}

export class DeserializeError extends Error {
    constructor(error: Error) {
        super(error.message);
        this.name = 'DeserializeError';
    }
}

export type UnexpectedCode = number;

export async function callApi<ResponseBody, ResponseOk, ResponseCode>(
    url: string,
    init: FetchRequestInit,
    returnRespCodeNotOk: (code: number) => UnexpectedCode | ResponseCode,
    returnResponseOk: (deserializedBody: ResponseBody) => UnexpectedBodyError | ResponseOk
): Promise<
    ResponseOk | ResponseCode | UnexpectedCode | UnexpectedBodyError | FetchError
> {
    let resp;
    try {
        console.log(`api called for: fetch(${url}, ${init})`);
        resp = await fetch(url, init);
        console.log(`api res: ${resp}`);
        if (!resp.ok) return returnRespCodeNotOk(resp.status);
    } catch (e) {
        if (e instanceof Error) return new FetchError(e);
        else
            return new FetchError(new Error('Unexpected non Error object thrown from fetch'));
    }

    let resp_json: ResponseBody;
    try {
        resp_json = await resp.json();
    } catch (e) {
        if (e instanceof Error) return new FetchError(e);
        else
            return new FetchError(
                new Error('Unexpected non Error object thrown from FetchResponse.json()')
            );
    }

    return returnResponseOk(resp_json);
}
