import { ApiId } from '@/bindings/ApiId';
import {
  PostUserSummaryRequestBody,
  PostUserSummaryResponseBody,
  PostUserSummaryResponseCode,
} from '@/bindings/endpoints/PostUserSummary';
import { FetchRequestInit } from 'expo/fetch';
import {
  BASE,
  callApi,
  DeserializeError,
  FetchError,
  UnexpectedBodyError,
  UnexpectedCode,
} from './common';

function matchResponseCodeNotOk(
  resp_code: number
): PostUserSummaryResponseCode | UnexpectedCode {
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

export async function fetchUserSummary(
  username: string,
  api_id: ApiId
): Promise<
  | PostUserSummaryResponseCode
  | UnexpectedCode
  | PostUserSummaryResponseBody
  | UnexpectedBodyError
  | FetchError
  | DeserializeError
> {
  const body: PostUserSummaryRequestBody = {
    username: username,
    user_api_id: api_id,
  };

  const INIT: FetchRequestInit = {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(body),
  };

  const returnResponseOk = (resp_json: PostUserSummaryResponseBody) => {
    if (resp_json && resp_json.summary) {
      return resp_json;
    } else {
      return new UnexpectedBodyError('PostUserSummaryResponseBody inconsistent');
    }
  };

  return await callApi(
    BASE + '/api/v0/post_user_summary',
    INIT,
    matchResponseCodeNotOk,
    returnResponseOk
  );
}
