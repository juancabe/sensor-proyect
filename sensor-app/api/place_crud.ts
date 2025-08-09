import { ApiId } from '@/bindings/ApiId';
import {
    PostPlaceRequestBody,
    PostPlaceResponseBody,
    PostPlaceResponseCode,
} from '@/bindings/endpoints/PostPlace';
import { PlaceColor } from '@/bindings/PlaceColor';
import { FetchRequestInit } from 'expo/fetch';
import {
    BASE,
    DeserializeError,
    FetchError,
    UnexpectedBodyError,
    UnexpectedCode,
    callApi,
} from './common';

function matchResponseCodeNotOk(
    resp_code: number
): PostPlaceResponseCode | UnexpectedCode {
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

export async function deleteUserPlace(
    user_api_id: ApiId,
    place_id: ApiId
): Promise<
    | PostPlaceResponseCode
    | UnexpectedCode
    | boolean
    | UnexpectedBodyError
    | FetchError
    | DeserializeError
> {
    const body: PostPlaceRequestBody = {
        'Delete': {
            user_api_id,
            place_id,
        },
    };

    const INIT: FetchRequestInit = {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(body),
    };

    const returnResponseOk = (resp_json: PostPlaceResponseBody) => {
        return true;
    };

    return await callApi(
        BASE + '/api/v0/post_place',
        INIT,
        matchResponseCodeNotOk,
        returnResponseOk
    );
}

export async function newUserPlace(
    username: string,
    user_api_id: ApiId,
    place_name: string,
    place_description: string | null,
    place_color: PlaceColor
): Promise<
    | PostPlaceResponseCode
    | UnexpectedCode
    | {
          place_id: ApiId;
          place_name: string;
          place_description: string | null;
      }
    | UnexpectedBodyError
    | FetchError
    | DeserializeError
> {
    const body: PostPlaceRequestBody = {
        'Create': {
            username,
            user_api_id,
            place_name,
            place_description,
            place_color,
        },
    };

    const INIT: FetchRequestInit = {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(body),
    };

    const returnResponseOk = (resp_json: PostPlaceResponseBody) => {
        if (typeof resp_json === 'object' && 'Created' in resp_json) {
            return resp_json.Created;
        } else {
            return new UnexpectedBodyError('PostUserSummaryResponseBody inconsistent');
        }
    };

    return await callApi(
        BASE + '/api/v0/post_place',
        INIT,
        matchResponseCodeNotOk,
        returnResponseOk
    );
}
