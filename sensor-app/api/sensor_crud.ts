import { ApiId } from '@/bindings/ApiId';
import {
    PostSensorRequestBody,
    PostSensorResponseBody,
    PostSensorResponseCode,
} from '@/bindings/endpoints/PostSensor';
import { SensorColor } from '@/bindings/SensorColor';
import { SensorKind } from '@/bindings/SensorKind';
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
): PostSensorResponseCode | UnexpectedCode {
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

export async function deleteUserSensor(
    user_api_id: ApiId,
    sensor_api_id: ApiId
): Promise<
    | PostSensorResponseCode
    | UnexpectedCode
    | boolean
    | UnexpectedBodyError
    | FetchError
    | DeserializeError
> {
    const body: PostSensorRequestBody = {
        'DeleteSensor': {
            user_api_id,
            sensor_api_id,
        },
    };

    const INIT: FetchRequestInit = {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(body),
    };

    const returnResponseOk = (resp_json: PostSensorResponseBody) => {
        if (!(resp_json && resp_json.sensor_api_id))
            console.warn('Unexpected body received');

        return true;
    };

    return await callApi(
        BASE + '/api/v0/post_sensor',
        INIT,
        matchResponseCodeNotOk,
        returnResponseOk
    );
}

export async function newUserSensor(
    user_api_id: ApiId,
    user_place_id: ApiId,
    device_id: ApiId,
    sensor_kind: SensorKind,
    sensor_name: string,
    sensor_description: string | null,
    sensor_color: SensorColor
): Promise<
    | PostSensorResponseCode
    | UnexpectedCode
    | PostSensorResponseBody
    | UnexpectedBodyError
    | FetchError
    | DeserializeError
> {
    const body: PostSensorRequestBody = {
        'CreateSensor': {
            user_api_id,
            user_place_id,
            device_id,
            sensor_kind,
            sensor_name,
            sensor_description,
            sensor_color,
        },
    };

    const INIT: FetchRequestInit = {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(body),
    };

    const returnResponseOk = (resp_json: PostSensorResponseBody) => {
        if (resp_json && resp_json.sensor_api_id) {
            return resp_json;
        } else {
            return new UnexpectedBodyError('PostUserSummaryResponseBody inconsistent');
        }
    };

    return await callApi(
        BASE + '/api/v0/post_sensor',
        INIT,
        matchResponseCodeNotOk,
        returnResponseOk
    );
}
