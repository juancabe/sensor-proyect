import { equidistantIndices } from '@/helpers/equidistantIndices';
import { safeGet } from '@/helpers/objectWork';
import { timeDisplay } from '@/helpers/timeDisplay';

export type UNIXTimestampSeconds = number;
export type TimestampedData = [object, UNIXTimestampSeconds][];

export interface DataInsights {
    minValue: number;
    maxValue: number;
}

export interface DataShape {
    maxLabels: number;
    maxPoints: number;
    significantFigures?: number;
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
        const maxPoints = dataShape.maxPoints;
        const maxLabels = dataShape.maxLabels;

        // Indices of the data that will be represented
        const workingIndices = equidistantIndices(data.length, maxPoints);

        // Data that will be represented
        let workingData: TimestampedData[] = [];
        for (let index in workingIndices) {
            workingData.push(data[index] as TimestampedData);
        }

        // Indices of the data that will contain labels
        let labeledIndices = equidistantIndices(workingData.length, maxLabels);

        // Array that will be filled with the corresponding ShapedDatumArray for each key
        const shapedData: ShapedDatumArray[] = keys.map((key) => {
            return {
                key: key,
                array: [],
                dataInsights: { maxValue: 0, minValue: Number.MAX_SAFE_INTEGER },
            };
        });

        labeledIndices = labeledIndices.reverse();

        for (const [index, datum] of workingData.entries()) {
            const nextLabeled = labeledIndices.at(labeledIndices.length - 1);

            let labeled = false;
            if (nextLabeled === index) {
                labeledIndices.pop();
                labeled = true;
            }

            for (let i = 0; i < shapedData.length; i++) {
                const newDatum = safeGet(datum[0], keys[i]);
                if (!newDatum) {
                    throw `newDatum should exist, not existing means the key ${keys[i]} isnt common to all datums, this datum is: ${Object.keys(datum)}`;
                }
                if (!(typeof newDatum === 'number')) {
                    throw 'newDatum should be a number';
                }

                const label =
                    labeled && typeof datum[1] === 'number'
                        ? timeDisplay(new Date(datum[1] * 1000), false, true)
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
