import { ApiId } from '@/bindings/ApiId';
import { FetchRequestInit } from 'expo/fetch';
import {
    BASE,
    DeserializeError,
    FetchError,
    UnexpectedBodyError,
    UnexpectedCode,
    callApi,
} from './common';
import {
    GetSensorDataRequestBody,
    GetSensorDataResponseBody,
    GetSensorDataResponseCode,
} from '@/bindings/endpoints/GetSensorData';

function matchResponseCodeNotOk(
    resp_code: number,
): GetSensorDataResponseCode | UnexpectedCode {
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

export async function getSensorData(
    user_api_id: ApiId,
    sensor_api_id: ApiId,
    added_at_lower: number | null,
    added_at_upper: number | null,
): Promise<
    | GetSensorDataResponseCode
    | UnexpectedCode
    | GetSensorDataResponseBody
    | UnexpectedBodyError
    | FetchError
    | DeserializeError
> {
    const body: GetSensorDataRequestBody = {
        user_api_id,
        sensor_api_id,
        added_at_upper,
        added_at_lower,
    };

    const INIT: FetchRequestInit = {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(body),
    };

    const returnResponseOk = (resp_json: GetSensorDataResponseBody) => {
        if (resp_json && 'item_count' in resp_json) {
            return resp_json;
        } else {
            return new UnexpectedBodyError('GetSensorDataResponseBody inconsistent');
        }
    };

    return await callApi(
        BASE + '/api/v0/get_sensor_data',
        INIT,
        matchResponseCodeNotOk,
        returnResponseOk,
    );
}
