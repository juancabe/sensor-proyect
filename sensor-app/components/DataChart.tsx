import { equidistantIndices } from '@/helpers/equidistantIndices';
import { toSignificantFigures } from '@/helpers/significantFigures';
import { ShapedDatumArray } from '@/model/ShapedData';
import { Box, Theme } from '@/ui/theme';
import { useTheme } from '@shopify/restyle';
import { LineChart } from 'react-native-gifted-charts';

export interface DataChartProps {
    data: ShapedDatumArray;
    params: ChartParams;
    padding: number;
}

export function calculateParams(
    keyData: ShapedDatumArray,
    width: number,
    noOfYAxisSections: number,
    significantFigures: number,
): ChartParams {
    //-- y axis related
    const minVal = keyData.dataInsights.minValue;
    const maxVal = keyData.dataInsights.maxValue;

    console.log('minVal: ', minVal);
    console.log('maxVal: ', maxVal);

    const minMaxDiff = maxVal - minVal;
    const padding = minMaxDiff / 3;

    const yAxisOffset = minVal - padding;
    const maxValue = maxVal - yAxisOffset + padding;
    const minValue = minVal;

    console.log('calculateParams noOfYAxisSections: ', noOfYAxisSections);
    let yAxisLabelNumbers = equidistantIndices(
        keyData.array.length,
        noOfYAxisSections - 1,
    )
        .map(
            (desiredIndex) =>
                [...keyData.array].sort((first, latter) => first.value - latter.value)[
                    desiredIndex
                ],
        )
        .map((shapedDatum) => shapedDatum.value);
    let yAxisLabelTexts = [
        yAxisLabelNumbers[0] - padding,
        ...yAxisLabelNumbers,
        yAxisLabelNumbers.at(yAxisLabelNumbers.length - 1)! + padding,
    ].map((numericalValue) => toSignificantFigures(numericalValue, significantFigures));

    console.log('calculateParams yAxisLabelTexts: ', yAxisLabelTexts);

    //-- x axis
    const spacing = width / keyData.array.length;
    const yAxisLabelWidth = significantFigures * 8;
    //--

    return {
        maxRepresentedValue: maxValue,
        minValue,
        yAxisOffset,
        yAxisLabelWidth,
        yAxisLabelTexts,
        noOfYAxisSections,
        spacing,
        width,
    };
}

export interface ChartParams {
    //-- y axis
    maxRepresentedValue: number;
    minValue: number;
    yAxisOffset: number;
    yAxisLabelWidth: number;
    yAxisLabelTexts: string[];
    noOfYAxisSections: number;
    //-- x axis
    spacing: number;

    //-- general
    width: number;
}

export default function DataChart({ data, params, padding }: DataChartProps) {
    const theme = useTheme<Theme>();
    const yAxisColor = theme.colors.detail;

    console.log(
        `dataMaxValue(${data.key}): ${Math.max(...data.array.map((val) => val.value))})`,
    );

    return (
        <Box
            style={{
                paddingRight: padding,
            }}
        >
            <LineChart
                // adjustToWidth={true}
                dataPointsColor={theme.colors.text}
                verticalLinesColor={theme.colors.text}
                data={data.array}
                maxValue={params.maxRepresentedValue}
                noOfSections={params.noOfYAxisSections}
                spacing={params.spacing}
                hideRules
                color={yAxisColor}
                yAxisColor={yAxisColor}
                yAxisOffset={params.yAxisOffset}
                yAxisLabelWidth={params.yAxisLabelWidth}
                showYAxisIndices
                yAxisIndicesColor={yAxisColor}
                yAxisIndicesWidth={10}
                yAxisTextStyle={{
                    color: theme.colors.text,
                    overflow: 'visible',
                    minWidth: 35,
                    marginRight: 5,
                }}
                yAxisLabelTexts={params.yAxisLabelTexts}
                xAxisColor={theme.colors.text}
                xAxisIndicesColor={theme.colors.text}
                xAxisLabelTextStyle={{
                    color: theme.colors.text,
                    padding: 0,
                    marginLeft: -10,
                    minWidth: 80,
                    fontSize: 12,
                    overflow: 'hidden',
                    textAlign: 'left',
                }}
                xAxisIndicesHeight={10}
                xAxisIndicesWidth={2}
            />
        </Box>
    );
}
