import { useAppContext } from '@/components/AppProvider';
import CheckboxesSelector from '@/components/CheckboxesSelector';
import DataChart, { calculateParams } from '@/components/DataChart';
import LoadingScreen from '@/components/LoadingScreen';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import { hex6WithAlpha } from '@/helpers/withAlpha';
import useSensorDataApi from '@/hooks/useSensorDataApi';
import { Card } from '@/ui/components/Card';
import { Box, Text } from '@/ui/theme';
import { Redirect } from 'expo-router';
import { useEffect, useState } from 'react';
import { StyleSheet } from 'react-native';
import { useSafeAreaFrame } from 'react-native-safe-area-context';

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
    const hookedFrame = useSafeAreaFrame();
    const sensor = ctx.activeSensor;

    const [selectedKey, setSelectedKey] = useState<string | null>(null);
    const [selectedInterval, setSelectedInterval] = useState<string>(
        TimeInterval.HalfHour,
    );
    const [convertedInterval, setConvertedInterval] = useState<number>(30 * 60 * 1000);

    const { reload, availableKeys, data } = useSensorDataApi(convertedInterval, sensor);

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

    if (!data) {
        return <LoadingScreen></LoadingScreen>;
    }

    if (!sensor) {
        return <Redirect href={'/'} />;
    }

    const keyData = data.data.filter((d) => d.key === selectedKey).at(0);
    const chartParams = keyData
        ? calculateParams(keyData, hookedFrame.width - styles.chartContainer.padding * 12)
        : null;

    console.log('sensor.color', sensor.color);
    const secondaryColor = hex6WithAlpha(sensor.color, 0.33);
    console.log('secondaryColor', secondaryColor);

    return (
        <BackgroundView>
            <Card variant="elevated">
                <Text variant="body">Data for sensor</Text>
                <Text variant="heading">&apos;{sensor!.name}&apos;</Text>
            </Card>

            <Box>
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
                    <Text variant="heading" color="warning">
                        No keys available!
                    </Text>
                )}
                <Card variant="elevated">
                    {chartParams && keyData ? (
                        <DataChart data={keyData} params={chartParams} />
                    ) : (
                        <Text variant="heading">Select some data key!</Text>
                    )}
                </Card>
                <CheckboxesSelector
                    title="Time interval"
                    selectedValue={selectedInterval}
                    onValueChange={(v) => {
                        setSelectedInterval(v);
                    }}
                    values={Object.values(TimeInterval) as string[]}
                    style={{ margin: 10 }}
                />
            </Box>
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
