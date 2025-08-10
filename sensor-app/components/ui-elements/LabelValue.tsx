import { StyleSheet } from 'react-native';
import { ThemedText } from './ThemedText';
import { ThemedView } from './ThemedView';

// New props interface
export interface LabelValueProps {
    label: string;
    children: React.ReactNode; // Use children instead of values
    horizontal?: boolean;
}

export default function LabelValue(props: LabelValueProps) {
    return (
        <ThemedView
            style={[
                styles.labelValueContainer,
                props.horizontal
                    ? { flexDirection: 'row' }
                    : {
                          flexDirection: 'column',
                      },
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
        backgroundColor: '#00000040',
        padding: 10,
        borderRadius: 10,
        gap: 10,
    },
    label: {
        fontSize: 15,
        fontWeight: 'bold',
    },
});
