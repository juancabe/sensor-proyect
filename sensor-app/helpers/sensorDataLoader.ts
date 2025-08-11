import { getSensorData } from '@/api/get_sensor_data';
import { ApiId } from '@/bindings/ApiId';
import { GetSensorDataResponseBody } from '@/bindings/endpoints/GetSensorData';
import { DataShape, ShapedData, TimestampedData } from '@/model/ShapedData';

export interface ApiArgs {
    user_api_id: ApiId;
    sensor_api_id: ApiId;
    first: null | Date;
    last: null | Date;
}

export class SensorDataLoader {
    private _data: TimestampedData;
    private _numerical_keys: string[];
    private _data_shape: DataShape;
    private _shaped_data: ShapedData;

    private constructor(
        data: TimestampedData,
        numerical_keys: string[],
        data_shape: DataShape,
    ) {
        this._data = data;
        this._numerical_keys = numerical_keys;
        this._data_shape = data_shape;
        if (this._numerical_keys)
            this._shaped_data = ShapedData.load(data_shape, data, numerical_keys);
        else this._shaped_data = ShapedData.default();
    }

    getData(): ShapedData {
        return this._shaped_data;
    }

    getKeys(): string[] {
        return this._numerical_keys;
    }

    // Will throw string describing error
    static async load(apiArgs: ApiArgs, dataShape: DataShape): Promise<SensorDataLoader> {
        // TIP: ~~ operator to get rid of decimals
        const firstSecs = apiArgs.first ? ~~(apiArgs.first.getTime() / 1000) : null;
        const lastSecs = apiArgs.last ? ~~(apiArgs.last.getTime() / 1000) : null;

        if (firstSecs && lastSecs && firstSecs >= lastSecs) {
            throw 'Invalid time interval';
        }

        const ret = await getSensorData(
            apiArgs.user_api_id,
            apiArgs.sensor_api_id,
            firstSecs,
            lastSecs,
        );

        if (
            typeof ret !== 'object' ||
            (typeof ret === 'object' && !('item_count' in ret))
        ) {
            console.error(
                '[SensorDetail] loadSensorData error calling getSensorData: ',
                ret,
            );
            throw 'Error while fetching the data...';
        } else {
            console.log('GetSensorData returned correctly: ', typeof ret);
        }
        const tData = (ret as GetSensorDataResponseBody).serialized_data;
        let parsedData: [any, number][] = [];

        let commonNumberKeys: string[] | undefined = undefined;

        for (const [datum, date] of tData) {
            const parsed = JSON.parse(datum);
            parsedData.push([parsed, date]);

            const numberKeys = Object.entries(parsed)
                .filter(([, v]) => typeof v === 'number')
                .map(([k]) => k as string);

            if (commonNumberKeys === undefined) {
                commonNumberKeys = numberKeys;
            } else {
                commonNumberKeys = commonNumberKeys.filter((key) =>
                    numberKeys.includes(key),
                );
            }
        }

        if (!commonNumberKeys) {
            commonNumberKeys = [];
        }

        return new SensorDataLoader(parsedData, commonNumberKeys, dataShape);
    }
}
