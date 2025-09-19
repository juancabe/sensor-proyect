import { ShapedDatumArray } from '@/model/ShapedData';
import { Theme } from '@/ui/theme';
import { useTheme } from '@shopify/restyle';
import { LineChart } from 'react-native-gifted-charts';

export interface DataChartProps {
    data: ShapedDatumArray;
    params: ChartParams;
}

export function calculateParams(keyData: ShapedDatumArray, width: number): ChartParams {
    const minVal = keyData?.dataInsights.minValue;
    const maxVal = keyData?.dataInsights.maxValue;

    const minMaxDiff = maxVal - minVal;
    const tailSize = minMaxDiff / 4;

    const yAxisOffset = minVal - tailSize;
    const maxValue = maxVal - yAxisOffset + tailSize;

    const spacing = width / keyData.array.length;

    return {
        maxValue,
        yAxisOffset,
        spacing,
    };
}

export interface ChartParams {
    maxValue: number;
    yAxisOffset: number;
    spacing: number;
}

export default function DataChart({ data, params }: DataChartProps) {
    const theme = useTheme<Theme>();

    return (
        <LineChart
            dataPointsColor={theme.colors.text}
            verticalLinesColor={theme.colors.text}
            data={data?.array}
            maxValue={params?.maxValue}
            noOfSections={3}
            spacing={params?.spacing}
            hideRules
            color="orange"
            yAxisColor={'orange'}
            yAxisOffset={params?.yAxisOffset}
            showYAxisIndices
            yAxisIndicesColor={'orange'}
            yAxisIndicesWidth={10}
            yAxisTextStyle={{
                color: theme.colors.text,
            }}
            xAxisColor={theme.colors.text}
            xAxisIndicesColor={theme.colors.text}
            xAxisLabelTextStyle={{
                color: theme.colors.text,
                width: 80,
                marginLeft: -26,
            }}
            xAxisIndicesHeight={10}
            xAxisIndicesWidth={2}
        />
    );
}
