import { ApiSensorData } from '@/bindings/api/endpoints/sensor_data/ApiSensorData';
import { DataShape, ShapedData, TimestampedData } from '@/model/ShapedData';

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
    static async load(
        gotData: ApiSensorData[],
        dataShape: DataShape,
    ): Promise<SensorDataLoader> {
        let parsedData: [any, number][] = [];

        let commonNumberKeys: string[] | undefined = undefined;

        for (const { data, added_at } of gotData) {
            const parsed = JSON.parse(JSON.parse(data));
            parsedData.push([parsed, added_at]);

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

        parsedData.reverse();

        if (!commonNumberKeys) {
            commonNumberKeys = [];
        }

        return new SensorDataLoader(parsedData, commonNumberKeys, dataShape);
    }
}
