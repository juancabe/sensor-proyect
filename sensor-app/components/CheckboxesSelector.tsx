import { StyleSheet, Text, TouchableOpacity, View } from 'react-native';
import { ThemedView } from './ui-elements/ThemedView';
import { TEXT_STYLES, ThemedText } from './ui-elements/ThemedText';

interface CheckboxesSelectorProps {
    selectedValue: string | null;
    onValueChange: (value: string) => void;
    values: string[];
    title?: string;
    style?: object;
}

export default function CheckboxesSelector({
    selectedValue,
    onValueChange,
    values,
    title,
    style,
}: CheckboxesSelectorProps) {
    return (
        <ThemedView style={styles.mainContainer}>
            {title ? <ThemedText style={TEXT_STYLES.heading2}>{title}</ThemedText> : null}
            <ThemedView style={[style, styles.container]}>
                {values.map((value) => {
                    const isSelected = selectedValue === value;
                    const isSelectedBorder = isSelected
                        ? styles.selectedBorder
                        : undefined;

                    return (
                        <View key={value} style={styles.valueContainer}>
                            <ThemedText>{value}</ThemedText>
                            <TouchableOpacity
                                style={[styles.checkBox, isSelectedBorder]}
                                onPress={() => onValueChange(value)}
                            >
                                {isSelected ? <Text style={styles.tick}>✔</Text> : null}
                            </TouchableOpacity>
                        </View>
                    );
                })}
            </ThemedView>
        </ThemedView>
    );
}

const styles = StyleSheet.create({
    tick: {
        color: '#AAF',
        fontSize: 30,
    },
    mainContainer: {
        display: 'flex',
        flexDirection: 'column',
        alignContent: 'center',
        borderColor: '#FFF',
        borderWidth: 3,
        borderRadius: 10,
        padding: 10,
        gap: 6,
    },
    container: {
        display: 'flex',
        flexWrap: 'wrap',
        flexDirection: 'row',
        alignContent: 'center',
        justifyContent: 'space-around',
        backgroundColor: '#000',
        gap: 6,
    },
    checkBox: {
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        width: 40,
        height: 40,
        borderRadius: 5,
        borderColor: '#00A',
        backgroundColor: '#222299aa',
        borderWidth: 3,
    },
    selectedBorder: {
        borderColor: '#00F',
        borderWidth: 5,
        backgroundColor: '#2222CCaa',
    },
    valueContainer: {
        display: 'flex',
        flexDirection: 'row',
        alignItems: 'center',
        gap: 10,
        borderColor: '#7777bb99',
        borderWidth: 3,
        backgroundColor: '#001155FF',
        padding: 5,
        borderRadius: 10,
    },
});
