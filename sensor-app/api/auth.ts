import { ApiId } from '@/bindings/ApiId';
import {
  GetLoginRequestBody,
  GetLoginResponseBody,
  GetLoginResponseCode,
} from '@/bindings/endpoints/GetLogin';
import {
  RegisterIncorrectReason,
  RegisterRequestBody,
  RegisterResponseBody,
  RegisterResponseCode,
} from '@/bindings/endpoints/Register';
import { fetch, FetchRequestInit } from 'expo/fetch';

const BASE = 'http://sensor-server.juancb.ftp.sh:3000';

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

function matchResponseCodeNotOk(
  resp_code: number
): GetLoginResponseCode | RegisterResponseCode | UnexpectedCode {
  switch (resp_code) {
    case 400:
      return 'BadRequest';
    case 413:
      return 'PayloadTooLarge';
    case 401:
      return 'Unauthorized';
    case 500:
      return 'InternalServerError';
    default:
      return resp_code;
  }
}

async function callApi<RequestBody, ResponseBody, ResponseOk, ResponseCode>(
  url: string,
  init: FetchRequestInit,
  props: RequestBody,
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

export async function login(
  props: GetLoginRequestBody
): Promise<
  ApiId | GetLoginResponseCode | UnexpectedCode | UnexpectedBodyError | FetchError
> {
  const PATH = '/api/v0/login';
  const INIT: FetchRequestInit = {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(props),
  };
  const returnResponseOk = (resp_json: GetLoginResponseBody) => {
    if (resp_json && resp_json.api_id) {
      return resp_json.api_id;
    } else {
      return new UnexpectedBodyError('api_id missing in response');
    }
  };

  return callApi(BASE + PATH, INIT, props, matchResponseCodeNotOk, returnResponseOk);
}

export async function register(
  props: RegisterRequestBody
): Promise<
  | ApiId
  | RegisterResponseCode
  | RegisterIncorrectReason
  | UnexpectedCode
  | UnexpectedBodyError
  | FetchError
> {
  const PATH = '/api/v0/register';
  const INIT: FetchRequestInit = {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(props),
  };

  const returnResponseOk = (resp_json: RegisterResponseBody) => {
    if ('Correct' in resp_json) {
      const apiId = resp_json.Correct;
      return apiId;
    } else if ('Incorrect' in resp_json) {
      const reason = resp_json.Incorrect;
      return reason;
    } else {
      return new UnexpectedBodyError('RegisterResponseBody inconsistent');
    }
  };

  return callApi(BASE + PATH, INIT, props, matchResponseCodeNotOk, returnResponseOk);
}
