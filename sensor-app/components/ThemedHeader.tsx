import { TEXT_STYLES, ThemedText } from './ui-elements/ThemedText';
import { StyleSheet, View } from 'react-native';
import useLayerColor from '@/hooks/useLayerColor';

export interface HeaderProps {
    children: React.ReactNode;
}

export default function ThemedHeader(props: HeaderProps) {
    const backgroundColor = useLayerColor();

    return (
        <View style={[styles.headerContainer]}>
            <ThemedText
                style={[
                    TEXT_STYLES.heading1,
                    { backgroundColor: backgroundColor },
                    styles.text,
                ]}
            >
                {props.children}
            </ThemedText>
        </View>
    );
}

const styles = StyleSheet.create({
    headerContainer: {
        flexDirection: 'row',
        padding: 25,
        marginTop: 5,
        marginBottom: 5,
    },
    text: {
        padding: 10,
        borderRadius: 10,
    },
});
