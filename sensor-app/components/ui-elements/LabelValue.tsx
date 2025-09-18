import { StyleSheet } from 'react-native';
import { Card } from '@/ui/components/Card';
import { Box, Text } from '@/ui/theme';
import { BoxV } from '@/ui/components/BoxV';

// New props interface
export interface LabelValueProps {
    label: string;
    children: React.ReactNode; // Use children instead of values
    horizontal?: boolean;
}

export default function LabelValue(props: LabelValueProps) {
    return (
        <BoxV
            variant="field"
            backgroundColor="keyBackground"
            flexDirection={props.horizontal ? 'row' : 'column'}
            alignItems="center"
            style={{ padding: 0 }}
        >
            <Text variant="body" paddingRight="m" paddingLeft="m">
                {props.label}
            </Text>
            <BoxV variant="valueCell">{props.children}</BoxV>
        </BoxV>
    );
}

const styles = StyleSheet.create({
    labelValueContainer: {
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'center',
        alignItems: 'center',
        padding: 10,
        borderRadius: 10,
        gap: 10,
    },
    label: {
        fontSize: 15,
    },
});
