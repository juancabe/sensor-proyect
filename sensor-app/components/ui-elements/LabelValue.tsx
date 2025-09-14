import { StyleSheet } from 'react-native';
import { ThemedText } from './ThemedText';
import { ThemedView } from './ThemedView';
import { useTheme } from '@react-navigation/native';
import useLayerColor from '@/hooks/useLayerColor';

// New props interface
export interface LabelValueProps {
    label: string;
    children: React.ReactNode; // Use children instead of values
    horizontal?: boolean;
}

export default function LabelValue(props: LabelValueProps) {
    const backgroundColor = useLayerColor();

    return (
        <ThemedView
            style={[
                styles.labelValueContainer,
                props.horizontal
                    ? { flexDirection: 'row' }
                    : {
                          flexDirection: 'column',
                      },
                { backgroundColor: backgroundColor },
            ]}
        >
            <ThemedText style={styles.label}>{props.label}</ThemedText>
            {props.children}
        </ThemedView>
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
