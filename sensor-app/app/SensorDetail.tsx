import { useAppContext } from '@/components/AppProvider';
import CheckboxesSelector from '@/components/CheckboxesSelector';
import LoadingScreen from '@/components/LoadingScreen';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import ErrorBox from '@/components/ui-elements/ErrorBox';
import { TEXT_STYLES, ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import { DataShape, ShapedDatumArray } from '@/model/ShapedData';
import { useCallback, useEffect, useState } from 'react';
import { SafeAreaView, StyleSheet } from 'react-native';
import { LineChart } from 'react-native-gifted-charts';
import { Rect, useSafeAreaFrame } from 'react-native-safe-area-context';

interface ChartParams {
    maxValue: number;
    yAxisOffset: number;
    spacing: number;
}

// TODO: Make it work again! (MIWA)

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

export default function SensorDetail() {
    const ctx = useAppContext();
    const frame = useSafeAreaFrame();
    const session = ctx.sessionData;
    const sensor = ctx.activeSensor;

    const [errorText, setErrorText] = useState<string | null>(null);
    // const [dataLoader, setDataLoader] = useState<SensorDataLoader | undefined>(undefined);
    const [availableKeys, setAvailableKeys] = useState<string[] | undefined>(undefined);
    const [selectedKey, setSelectedKey] = useState<string | null>(null);
    const [selectedInterval, setSelectedInterval] = useState<string>(
        TimeInterval.HalfHour,
    );

    const getData = useCallback(async () => {
        if (!session?.all_set() || !sensor) {
            setErrorText('The session is corrupted');
            return;
        }

        //     try {
        //         const last = new Date();
        //
        //         let diff;
        //
        //         switch (selectedInterval) {
        //             case TimeInterval.HalfHour:
        //                 diff = 30 * 60 * 1000;
        //                 break;
        //             case TimeInterval.Hour:
        //                 diff = 60 * 60 * 1000;
        //                 break;
        //             case TimeInterval.FourHour:
        //                 diff = 4 * 60 * 60 * 1000;
        //                 break;
        //             case TimeInterval.TwelveHour:
        //                 diff = 12 * 60 * 60 * 1000;
        //                 break;
        //             case TimeInterval.Day:
        //                 diff = 24 * 60 * 60 * 1000;
        //                 break;
        //             case TimeInterval.Week:
        //                 diff = 7 * 24 * 60 * 60 * 1000;
        //                 break;
        //             case TimeInterval.Month:
        //                 diff = 30 * 24 * 60 * 60 * 1000;
        //                 break;
        //             default:
        //                 throw 'Impossible!';
        //         }
        //
        //         const first = new Date(last.getTime() - diff);
        //
        //         const apiArgs: ApiArgs = {
        //             user_api_id: { id: session.api_id! },
        //             sensor_api_id: sensor.api_id,
        //             first,
        //             last,
        //         };
        //
        //         const dataShape: DataShape = {
        //             maxLabels: 4,
        //             maxPoints: 1000,
        //         };
        //
        //         try {
        //             const dl = await SensorDataLoader.load(apiArgs, dataShape);
        //             setDataLoader(dl);
        //             setAvailableKeys(dl.getKeys());
        //         } catch (e) {
        //             console.error('Error SensorDataLoader.load: ', e);
        //         }
        //     } catch (e) {
        //         console.warn('[SensorDetail] error on getData: ', e);
        //         if (typeof e === 'string') setErrorText(e);
        //     }
    }, [sensor, session, selectedInterval]);

    useEffect(() => {
        getData();
        const id = setInterval(() => {
            console.warn('getting data');
            getData();
        }, 1000);

        return () => clearInterval(id);
    }, [getData]);

    // const data = dataLoader?.getData().data;
    // if (!data) {
    //     return <LoadingScreen></LoadingScreen>;
    // }

    // const keyData = data.filter((d) => d.key === selectedKey).at(0);
    // const chartParams = keyData ? calculateParams(keyData, frame) : null;

    return (
        <BackgroundView secondaryColor={sensor!.color}>
            <SafeAreaView>
                <ThemedView style={styles.headerContainer}>
                    <ThemedText style={TEXT_STYLES.heading2}>Data for sensor</ThemedText>
                    <ThemedText style={TEXT_STYLES.heading1}>
                        &apos;{sensor!.name}&apos;
                    </ThemedText>
                </ThemedView>

                <ThemedView style={styles.mainContainer}>
                    {availableKeys && availableKeys.length > 0 ? (
                        <CheckboxesSelector
                            selectedValue={selectedKey}
                            onValueChange={(k) => {
                                setSelectedKey(k);
                            }}
                            values={availableKeys}
                            title={'Data Series'}
                        ></CheckboxesSelector>
                    ) : null}
                    {/* <ThemedView style={[styles.chartContainer]}> */}
                    {/*     {chartParams ? ( */}
                    {/*         <LineChart */}
                    {/*             dataPointsColor="white" */}
                    {/*             verticalLinesColor={'white'} */}
                    {/*             data={keyData?.array} */}
                    {/*             maxValue={chartParams?.maxValue} */}
                    {/*             noOfSections={3} */}
                    {/*             spacing={chartParams?.spacing} */}
                    {/*             hideRules */}
                    {/*             color="orange" */}
                    {/*             yAxisColor={'orange'} */}
                    {/*             yAxisOffset={chartParams?.yAxisOffset} */}
                    {/*             showYAxisIndices */}
                    {/*             yAxisIndicesColor={'orange'} */}
                    {/*             yAxisIndicesWidth={10} */}
                    {/*             yAxisTextStyle={{ */}
                    {/*                 color: 'white', */}
                    {/*             }} */}
                    {/*             xAxisColor={'white'} */}
                    {/*             xAxisIndicesColor={'white'} */}
                    {/*             xAxisLabelTextStyle={{ */}
                    {/*                 color: 'white', */}
                    {/*                 width: 80, */}
                    {/*                 marginLeft: -26, */}
                    {/*             }} */}
                    {/*             xAxisIndicesHeight={10} */}
                    {/*             xAxisIndicesWidth={2} */}
                    {/*         /> */}
                    {/*     ) : ( */}
                    {/*         <ThemedText style={TEXT_STYLES.heading2}> */}
                    {/*             Select some data key! */}
                    {/*         </ThemedText> */}
                    {/*     )} */}
                    {/* </ThemedView> */}
                    <CheckboxesSelector
                        title="Time interval"
                        selectedValue={selectedInterval}
                        onValueChange={(v) => {
                            setSelectedInterval(v);
                        }}
                        values={Object.values(TimeInterval) as string[]}
                    />
                    <ErrorBox error={errorText}></ErrorBox>
                </ThemedView>
            </SafeAreaView>
        </BackgroundView>
    );
}

const styles = StyleSheet.create({
    mainContainer: {
        display: 'flex',
        flexDirection: 'column',
        gap: 20,
        justifyContent: 'space-between',
        height: '100%',
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
