import { getSensorData } from '@/api/get_sensor_data';
import { ApiId } from '@/bindings/ApiId';
import { GetSensorDataResponseBody } from '@/bindings/endpoints/GetSensorData';
import { timeDisplay } from './timeDisplay';

function equidistantIndices(len: number, max: number): number[] {
    if (max <= 0 || len <= 0) {
        return [];
    }

    if (len === 1) {
        // Si len es 1 siempre sera [0]
        return [0];
    }

    let indices: number[];

    if (max === 1) {
        // Elemento central (mitad alta si N es par)
        indices = [Math.floor(len / 2)];
    } else {
        // Limitamos numero de indices a len, se puede no limitar
        max = Math.min(max, len);
        // Espaciado proporcional en el rango [0, N-1]
        const step = (len - 1) / (max - 1);
        indices = Array.from({ length: max }, (_, i) => Math.round(i * step));
    }

    return indices;
}

function safeGet<T extends object>(obj: T, key: string): T[keyof T] | undefined {
    return key in obj ? obj[key as keyof T] : undefined;
}

type UNIXTimestampSeconds = number;
type TimestampedData = [object, UNIXTimestampSeconds][];

export interface ApiArgs {
    user_api_id: ApiId;
    sensor_api_id: ApiId;
    first: null | Date;
    last: null | Date;
}

export interface DataShape {
    maxLabels: number;
    maxPoints: number;
}

export interface ShapedDatum {
    value: number;
    label?: string;
    showXAxisIndex?: boolean;
    hideDataPoint?: boolean;
    onPress?: Function;
}

export interface ShapedDatumArray {
    key: string;
    array: ShapedDatum[];
    dataInsights: DataInsights;
}

const MAX_DATUM_ARRAYS = 4;
export interface ShapedData {
    data: ShapedDatumArray[];
}

export interface DataInsights {
    minValue: number;
    maxValue: number;
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
        if (this._numerical_keys) this._shaped_data = this.createShapedData();
        else this._shaped_data = { data: [] };
    }

    private createShapedData(): ShapedData {
        console.log('creating shaped data');

        const maxPoints = this._data_shape.maxPoints;
        const maxLabels = this._data_shape.maxLabels;
        const workingIndices = equidistantIndices(this._data.length, maxPoints);
        const keys = this._numerical_keys!;

        console.debug(
            `generating shapedData with available ${keys} and ${workingIndices.length} points`,
        );

        let workingData: TimestampedData[] = [];
        for (let index in workingIndices) {
            workingData.push(this._data[index] as TimestampedData);
        }

        const datumArraysNum = Math.min(MAX_DATUM_ARRAYS, keys.length);
        let labeledIndices = equidistantIndices(workingData.length, maxLabels);

        let shapedData: ShapedData = {
            data: Array.from({ length: datumArraysNum }, (_, index) => {
                const sDArr = {
                    key: keys[index],
                    array: [],
                    dataInsights: { maxValue: 0, minValue: Number.MAX_SAFE_INTEGER },
                };
                return sDArr;
            }),
        };

        labeledIndices = labeledIndices.reverse();

        for (const [index, datum] of workingData.entries()) {
            const nextLabeled = labeledIndices.at(labeledIndices.length - 1);

            let labeled = false;
            if (nextLabeled === index) {
                labeledIndices.pop();
                labeled = true;
            }

            for (let i = 0; i < datumArraysNum; i++) {
                const newDatum = safeGet(datum[0], keys[i]);
                if (!newDatum) {
                    throw `newDatum should exist, not existing means the key ${keys[i]} isnt common to all datums, this datum is: ${Object.keys(datum)}`;
                }
                if (!(typeof newDatum === 'number')) {
                    throw 'newDatum should be a number';
                }

                const label =
                    labeled && typeof datum[1] === 'number'
                        ? timeDisplay(new Date(datum[1] * 1000))
                        : undefined;

                if (label) {
                    console.log('label for ', datum[1], ': ', label);
                }

                const shapedDatum: ShapedDatum = {
                    value: newDatum,
                    label,
                    showXAxisIndex: labeled,
                };

                shapedData.data[i].array.push(shapedDatum);
                shapedData.data[i].dataInsights.maxValue = Math.max(
                    newDatum,
                    shapedData.data[i].dataInsights.maxValue,
                );
                shapedData.data[i].dataInsights.minValue = Math.min(
                    newDatum,
                    shapedData.data[i].dataInsights.minValue,
                );
            }
        }

        console.debug(`returning shapedData`);

        return shapedData;
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
