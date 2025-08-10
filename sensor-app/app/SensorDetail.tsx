import { useAppContext } from '@/components/AppProvider';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import ErrorBox from '@/components/ui-elements/ErrorBox';
import { ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import { ApiArgs, DataShape, SensorDataLoader } from '@/helpers/sensorDataLoader';
import { useEffect, useState } from 'react';
import { SafeAreaView } from 'react-native';
import { LineChart } from 'react-native-gifted-charts';
import { LinearGradient, Stop } from 'react-native-svg';

export default function SensorDetail() {
    const ctx = useAppContext();
    const session = ctx.sessionData;
    const sensor = ctx.activeSensor;

    const [errorText, setErrorText] = useState<string | null>(null);
    const [dataLoader, setDataLoader] = useState<SensorDataLoader | undefined>(undefined);

    const data = dataLoader?.getData().data[0];
    console.log(`data: ${data?.array.map((v) => v.value)}`);

    const minVal = data?.dataInsights.minValue!;
    const maxVal = data?.dataInsights.maxValue!;

    const minMaxDiff = maxVal - minVal;
    const tailSize = minMaxDiff / 4;

    const yAxisOffset = minVal - tailSize;
    const maxValue = maxVal - yAxisOffset + tailSize;

    console.log(`maxValue: ${maxValue}, yAxisOffset: ${yAxisOffset}`);

    useEffect(() => {
        if (!session?.all_set() || !sensor) {
            setErrorText('The session is corrupted');
            return;
        }

        const getData = async () => {
            try {
                const last = new Date();
                const first = new Date(last.getTime() - 5000 * 60);
                // const first = null;
                // const last = null;

                const apiArgs: ApiArgs = {
                    user_api_id: { id: session.api_id! },
                    sensor_api_id: sensor.api_id,
                    first,
                    last,
                };

                const dataShape: DataShape = {
                    maxLabels: 1000,
                    maxPoints: 1000,
                };

                try {
                    const dl = await SensorDataLoader.load(apiArgs, dataShape);
                    setDataLoader(dl);
                } catch (e) {
                    console.error('Error SensorDataLoader.load: ', e);
                }
            } catch (e) {
                console.warn('[SensorDetail] error on getData: ', e);
                if (typeof e === 'string') setErrorText(e);
            }
        };

        getData();
    }, [session, sensor]);

    return (
        <BackgroundView secondaryColor={sensor!.color}>
            <SafeAreaView>
                <ErrorBox error={errorText}></ErrorBox>
                <ThemedText>This is the SensorDetail my G!!!!</ThemedText>
                <ThemedView style={{ backgroundColor: 'white' }}>
                    <LineChart
                        data={data?.array}
                        maxValue={maxValue}
                        noOfSections={3}
                        spacing={80}
                        hideRules
                        color="orange"
                        yAxisColor={'orange'}
                        yAxisOffset={yAxisOffset}
                        showYAxisIndices
                        yAxisIndicesColor={'orange'}
                        yAxisIndicesWidth={10}
                        xAxisLabelTextStyle={{
                            width: 80,
                            // marginLeft: -36
                        }}
                        xAxisIndicesHeight={10}
                        xAxisIndicesWidth={2}
                    />{' '}
                </ThemedView>
            </SafeAreaView>
        </BackgroundView>
    );
}
