import { ThemedText, TEXT_STYLES } from './ThemedText';
import { ThemedView } from './ThemedView';

export interface ErrorBoxProps {
    error: string | null;
    style?: object;
}

export default function ErrorBox({ error, style }: ErrorBoxProps) {
    return (
        <ThemedView
            style={[
                {
                    borderStyle: 'solid',
                    borderColor: '#ff0000',
                    borderWidth: 4,
                    borderRadius: 4,
                    backgroundColor: '#ff000066',
                    padding: 40,
                    opacity: error ? 100 : 0,
                },
                style,
            ]}
        >
            {error ? <ThemedText style={TEXT_STYLES.body}>{error}</ThemedText> : null}
        </ThemedView>
    );
}
