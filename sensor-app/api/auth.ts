import { ApiId } from '@/bindings/ApiId';
import {
  LoginRequestBody,
  LoginResponseBody,
  LoginResponseCode,
} from '@/bindings/endpoints/Login';
import {
  RegisterIncorrectReason,
  RegisterRequestBody,
  RegisterResponseBody,
  RegisterResponseCode,
} from '@/bindings/endpoints/Register';
import { FetchRequestInit } from 'expo/fetch';
import { BASE, callApi, FetchError, UnexpectedBodyError, UnexpectedCode } from './common';

function matchResponseCodeNotOk(
  resp_code: number
): LoginResponseCode | RegisterResponseCode | UnexpectedCode {
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

export async function login(
  props: LoginRequestBody
): Promise<
  ApiId | LoginResponseCode | UnexpectedCode | UnexpectedBodyError | FetchError
> {
  const PATH = '/api/v0/login';
  const INIT: FetchRequestInit = {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(props),
  };
  const returnResponseOk = (resp_json: LoginResponseBody) => {
    if (resp_json && resp_json.api_id) {
      return resp_json.api_id;
    } else {
      return new UnexpectedBodyError('api_id missing in response');
    }
  };

  return callApi(BASE + PATH, INIT, matchResponseCodeNotOk, returnResponseOk);
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

  return callApi(BASE + PATH, INIT, matchResponseCodeNotOk, returnResponseOk);
}
