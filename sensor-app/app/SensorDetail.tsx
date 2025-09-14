import { ApiUserSensor } from '@/bindings/api/endpoints/sensor/ApiUserSensor';
import { ApiSensorData } from '@/bindings/api/endpoints/sensor_data/ApiSensorData';
import { GetSensorData } from '@/bindings/api/endpoints/sensor_data/GetSensorData';
import { useAppContext } from '@/components/AppProvider';
import CheckboxesSelector from '@/components/CheckboxesSelector';
import LoadingScreen from '@/components/LoadingScreen';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import { TEXT_STYLES, ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import { SensorDataLoader } from '@/helpers/sensorDataLoader';
import { hex6WithAlpha } from '@/helpers/withAlpha';
import useApi from '@/hooks/useApi';
import useLayerColor from '@/hooks/useLayerColor';
import useRedirect from '@/hooks/useRedirect';
import { DataShape, ShapedDatumArray } from '@/model/ShapedData';
import { useTheme } from '@react-navigation/native';
import { useEffect, useMemo, useState } from 'react';
import { StyleSheet } from 'react-native';
import { LineChart } from 'react-native-gifted-charts';
import { Rect, useSafeAreaFrame } from 'react-native-safe-area-context';

interface ChartParams {
    maxValue: number;
    yAxisOffset: number;
    spacing: number;
}

function calculateParams(keyData: ShapedDatumArray, frame: Rect): ChartParams {
    const minVal = keyData?.dataInsights.minValue;
    const maxVal = keyData?.dataInsights.maxValue;

    const minMaxDiff = maxVal - minVal;
    const tailSize = minMaxDiff / 4;

    const yAxisOffset = minVal - tailSize;
    const maxValue = maxVal - yAxisOffset + tailSize;

    const spacing =
        (frame.width - styles.chartContainer.padding * 12) / keyData.array.length;

    return {
        maxValue,
        yAxisOffset,
        spacing,
    };
}

enum TimeInterval {
    HalfHour = '30m',
    Hour = '1h',
    FourHour = '4h',
    TwelveHour = '12h',
    Day = '1D',
    Week = '1W',
    Month = '1M',
}

const computeParams = (apiBody: GetSensorData | undefined) => {
    if (!apiBody) return;
    let params_arr = [['device_id', apiBody.device_id]];
    console.log('apiBody.lowest_added_at', apiBody.lowest_added_at);
    if (apiBody.lowest_added_at) {
        params_arr.push(['lowest_added_at', '' + apiBody.lowest_added_at]);
    }
    if (apiBody.upper_added_at) {
        params_arr.push(['upper_added_at', '' + apiBody.upper_added_at]);
    }
    return params_arr;
};

function fabGetSensorData(sensor: ApiUserSensor, offsetMillis: number): GetSensorData {
    return {
        device_id: sensor.device_id,
        lowest_added_at: ~~((Date.now() - offsetMillis) / 1000),
        upper_added_at: null,
    };
}

export default function SensorDetail() {
    const ctx = useAppContext();
    const frame = useSafeAreaFrame();
    const sensor = ctx.activeSensor;

    // const [errorText, setErrorText] = useState<string | null>(null);
    const [dataLoader, setDataLoader] = useState<SensorDataLoader | undefined>(undefined);
    const [availableKeys, setAvailableKeys] = useState<string[] | undefined>(undefined);
    const [selectedKey, setSelectedKey] = useState<string | null>(null);

    const [selectedInterval, setSelectedInterval] = useState<string>(
        TimeInterval.HalfHour,
    );
    const [convertedInterval, setConvertedInterval] = useState<number>(30 * 60 * 1000);

    const defaultApiBody: GetSensorData | undefined = sensor
        ? fabGetSensorData(sensor, convertedInterval)
        : undefined;
    const [apiBody, setApiBody] = useState<GetSensorData | undefined>(defaultApiBody);
    useEffect(() => {
        if (!sensor) return;
        setApiBody((prev) =>
            prev
                ? {
                      ...prev,
                      lowest_added_at: ~~((Date.now() - convertedInterval) / 1000),
                  }
                : fabGetSensorData(sensor, convertedInterval),
        );
    }, [sensor, convertedInterval]);

    const apiParams = useMemo(() => computeParams(apiBody), [apiBody]);

    const [apiMethod, setApiMethod] = useState<'GET' | undefined>('GET');
    const api = useApi('/sensor_data', apiMethod, false, undefined, apiParams);

    // work with received data
    useEffect(() => {
        if (!(api.returnedOk && api.response)) return;

        let cancelled = false;
        const load = async () => {
            const data = api.response as ApiSensorData[];
            const dataShape: DataShape = { maxLabels: 4, maxPoints: 1000 };
            const loader = await SensorDataLoader.load(data, dataShape);
            if (!cancelled) {
                setDataLoader(loader);
                console.warn('availableKeys: ', loader.getKeys());
                setAvailableKeys(loader.getKeys());
            }
        };

        load();
        return () => {
            cancelled = true;
        };
    }, [api.response, api.returnedOk]);

    useEffect(() => {
        const msByInterval: Record<string, number> = {
            [TimeInterval.HalfHour]: 30 * 60 * 1000,
            [TimeInterval.Hour]: 60 * 60 * 1000,
            [TimeInterval.FourHour]: 4 * 60 * 60 * 1000,
            [TimeInterval.TwelveHour]: 12 * 60 * 60 * 1000,
            [TimeInterval.Day]: 24 * 60 * 60 * 1000,
            [TimeInterval.Week]: 7 * 24 * 60 * 60 * 1000,
            [TimeInterval.Month]: 30 * 24 * 60 * 60 * 1000,
        };
        const diff = msByInterval[selectedInterval];
        setConvertedInterval(diff);
    }, [selectedInterval]);

    // useEffect(() => {
    //     const id = setInterval(() => {
    //         setApiMethod((prev) => {
    //             if (prev) {
    //                 return undefined;
    //             } else {
    //                 return 'GET';
    //             }
    //         });
    //     }, 3000);
    //
    //     return () => clearInterval(id);
    // }, []);
    //

    const redirect = useRedirect();
    const layerColor = useLayerColor();
    const theme = useTheme();

    const data = dataLoader?.getData().data;
    if (!data) {
        return <LoadingScreen></LoadingScreen>;
    }

    if (!sensor) {
        return redirect.redirectToIndex();
    }

    const keyData = data.filter((d) => d.key === selectedKey).at(0);
    const chartParams = keyData ? calculateParams(keyData, frame) : null;

    console.log('sensor.color', sensor.color);
    const secondaryColor = hex6WithAlpha(sensor.color, 0.33);
    console.log('secondaryColor', secondaryColor);

    return (
        <BackgroundView secondaryColor={secondaryColor}>
            <ThemedView style={styles.headerContainer}>
                <ThemedText style={TEXT_STYLES.heading2}>Data for sensor</ThemedText>
                <ThemedText style={TEXT_STYLES.heading1}>
                    &apos;{sensor!.name}&apos;
                </ThemedText>
            </ThemedView>

            <ThemedView style={[styles.mainContainer, { backgroundColor: layerColor }]}>
                {availableKeys && availableKeys.length > 0 ? (
                    <CheckboxesSelector
                        selectedValue={selectedKey}
                        onValueChange={(k) => {
                            setSelectedKey(k);
                        }}
                        values={availableKeys}
                        title={'Data Series'}
                        style={{ margin: 10 }}
                    ></CheckboxesSelector>
                ) : (
                    <ThemedText
                        style={[TEXT_STYLES.heading1, { justifyContent: 'center' }]}
                    >
                        No keys available!
                    </ThemedText>
                )}
                <ThemedView
                    style={[
                        styles.chartContainer,
                        { backgroundColor: theme.colors.background },
                    ]}
                >
                    {chartParams ? (
                        <LineChart
                            dataPointsColor={theme.colors.text}
                            verticalLinesColor={theme.colors.text}
                            data={keyData?.array}
                            maxValue={chartParams?.maxValue}
                            noOfSections={3}
                            spacing={chartParams?.spacing}
                            hideRules
                            color="orange"
                            yAxisColor={'orange'}
                            yAxisOffset={chartParams?.yAxisOffset}
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
                    ) : (
                        <ThemedText style={TEXT_STYLES.heading2}>
                            Select some data key!
                        </ThemedText>
                    )}
                </ThemedView>
                <CheckboxesSelector
                    title="Time interval"
                    selectedValue={selectedInterval}
                    onValueChange={(v) => {
                        setSelectedInterval(v);
                    }}
                    values={Object.values(TimeInterval) as string[]}
                    style={{ margin: 10 }}
                />
            </ThemedView>
        </BackgroundView>
    );
}

const styles = StyleSheet.create({
    mainContainer: {
        flex: 1,
        flexDirection: 'column',
        gap: 20,
        justifyContent: 'space-between',
        padding: 5,
        borderRadius: 15,
    },
    chartContainer: {
        display: 'flex',
        flexDirection: 'row',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 10,
        margin: -15,
    },
    headerContainer: {
        display: 'flex',
        flexDirection: 'row',
        justifyContent: 'center',
        alignItems: 'flex-end',
        backgroundColor: 'transparent',
        gap: 10,
    },
    checkBoxesContainer: {
        padding: 20,
    },
});
