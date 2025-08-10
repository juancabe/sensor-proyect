import { useAppContext } from '@/components/AppProvider';
import CheckboxesSelector from '@/components/CheckboxesSelector';
import LoadingScreen from '@/components/LoadingScreen';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import ErrorBox from '@/components/ui-elements/ErrorBox';
import { TEXT_STYLES, ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import {
    ApiArgs,
    DataShape,
    SensorDataLoader,
    ShapedData,
    ShapedDatumArray,
} from '@/helpers/sensorDataLoader';
import { useCallback, useEffect, useState } from 'react';
import { Button, SafeAreaView, StyleSheet } from 'react-native';
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

export default function SensorDetail() {
    const ctx = useAppContext();
    const frame = useSafeAreaFrame();
    const session = ctx.sessionData;
    const sensor = ctx.activeSensor;

    const [errorText, setErrorText] = useState<string | null>(null);
    const [dataLoader, setDataLoader] = useState<SensorDataLoader | undefined>(undefined);
    const [availableKeys, setAvailableKeys] = useState<string[] | undefined>(undefined);
    const [selectedKey, setSelectedKey] = useState<string | null>(null);

    const getData = useCallback(async () => {
        if (!session?.all_set() || !sensor) {
            setErrorText('The session is corrupted');
            return;
        }

        try {
            const last = new Date();
            const first = new Date(last.getTime() - 20_000 * 60);

            const apiArgs: ApiArgs = {
                user_api_id: { id: session.api_id! },
                sensor_api_id: sensor.api_id,
                first,
                last,
            };

            const dataShape: DataShape = {
                maxLabels: 4,
                maxPoints: 1000,
            };

            try {
                const dl = await SensorDataLoader.load(apiArgs, dataShape);
                setDataLoader(dl);
                setAvailableKeys(dl.getKeys());
            } catch (e) {
                console.error('Error SensorDataLoader.load: ', e);
            }
        } catch (e) {
            console.warn('[SensorDetail] error on getData: ', e);
            if (typeof e === 'string') setErrorText(e);
        }
    }, [sensor, session]);

    useEffect(() => {
        getData();
    }, [getData]);

    const data = dataLoader?.getData().data;
    if (!data) {
        return <LoadingScreen></LoadingScreen>;
    }

    const keyData = data.filter((d) => d.key === selectedKey).at(0);
    const chartParams = keyData ? calculateParams(keyData, frame) : null;

    return (
        <BackgroundView secondaryColor={sensor!.color}>
            <SafeAreaView>
                <ThemedView style={styles.mainContainer}>
                    <Button
                        title="Reload data"
                        onPress={() => {
                            getData();
                        }}
                    ></Button>
                    <ErrorBox error={errorText}></ErrorBox>
                    <ThemedView style={styles.headerContainer}>
                        <ThemedText style={TEXT_STYLES.heading2}>
                            Data for sensor
                        </ThemedText>
                        <ThemedText style={TEXT_STYLES.heading1}>
                            &apos;{sensor!.name}&apos;
                        </ThemedText>
                    </ThemedView>
                    {availableKeys ? (
                        <ThemedView style={styles.checkBoxesContainer}>
                            <CheckboxesSelector
                                selectedValue={selectedKey}
                                onValueChange={(k) => {
                                    setSelectedKey(k);
                                }}
                                values={availableKeys}
                            ></CheckboxesSelector>
                        </ThemedView>
                    ) : null}
                    <ThemedView style={[styles.chartContainer]}>
                        {chartParams ? (
                            <LineChart
                                dataPointsColor="white"
                                verticalLinesColor={'white'}
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
                                    color: 'white',
                                }}
                                xAxisColor={'white'}
                                xAxisIndicesColor={'white'}
                                xAxisLabelTextStyle={{
                                    color: 'white',
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
                </ThemedView>
            </SafeAreaView>
        </BackgroundView>
    );
}

const styles = StyleSheet.create({
    mainContainer: {
        display: 'flex',
        flexDirection: 'column',
    },
    chartContainer: {
        display: 'flex',
        flexDirection: 'row',
        alignItems: 'center',
        justifyContent: 'center',
        backgroundColor: '#222222aa',
        padding: 10,
        borderRadius: 10,
        margin: -15,
    },
    headerContainer: {
        display: 'flex',
        flexDirection: 'row',
        justifyContent: 'center',
        alignItems: 'flex-end',
        gap: 10,
    },
    checkBoxesContainer: {
        padding: 20,
    },
});
