import { safeGet } from '@/helpers/objectWork';
import { timeDisplay } from '@/helpers/timeDisplay';

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

export type UNIXTimestampSeconds = number;
export type TimestampedData = [object, UNIXTimestampSeconds][];

export interface DataInsights {
    minValue: number;
    maxValue: number;
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

export class ShapedData {
    private _data: ShapedDatumArray[];

    private constructor(data: ShapedDatumArray[]) {
        this._data = data;
    }

    get data(): ShapedDatumArray[] {
        return this._data;
    }

    static default() {
        return new ShapedData([]);
    }

    static load(dataShape: DataShape, data: TimestampedData, keys: string[]): ShapedData {
        console.log('creating shaped data');

        const maxPoints = dataShape.maxPoints;
        const maxLabels = dataShape.maxLabels;
        const workingIndices = equidistantIndices(data.length, maxPoints);

        console.debug(
            `generating shapedData with available ${keys} and ${workingIndices.length} points`,
        );

        let workingData: TimestampedData[] = [];
        for (let index in workingIndices) {
            workingData.push(data[index] as TimestampedData);
        }

        const datumArraysNum = Math.min(MAX_DATUM_ARRAYS, keys.length);
        let labeledIndices = equidistantIndices(workingData.length, maxLabels);

        let shapedData: ShapedDatumArray[] = Array.from(
            { length: datumArraysNum },
            (_, index) => {
                const sDArr = {
                    key: keys[index],
                    array: [],
                    dataInsights: { maxValue: 0, minValue: Number.MAX_SAFE_INTEGER },
                };
                return sDArr;
            },
        );

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

                const shapedDatum: ShapedDatum = {
                    value: newDatum,
                    label,
                    showXAxisIndex: labeled,
                };

                shapedData[i].array.push(shapedDatum);
                shapedData[i].dataInsights.maxValue = Math.max(
                    newDatum,
                    shapedData[i].dataInsights.maxValue,
                );
                shapedData[i].dataInsights.minValue = Math.min(
                    newDatum,
                    shapedData[i].dataInsights.minValue,
                );
            }
        }

        return new ShapedData(shapedData);
    }
}
