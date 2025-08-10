import { StyleSheet, Text, TouchableOpacity, View } from 'react-native';
import { ThemedView } from './ui-elements/ThemedView';
import { ThemedText } from './ui-elements/ThemedText';

interface CheckboxesSelectorProps {
    selectedValue: string | null;
    onValueChange: (color: string) => void;
    values: string[];
    style?: object;
}

export default function CheckboxesSelector({
    selectedValue,
    onValueChange,
    values,
    style,
}: CheckboxesSelectorProps) {
    console.log('values: ');
    return (
        <ThemedView style={[style, styles.container]}>
            {values.map((value) => {
                const isSelected = selectedValue === value;
                const isSelectedBorder = isSelected ? styles.selectedBorder : undefined;
                console.log(value);

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
    );
}

const styles = StyleSheet.create({
    tick: {
        color: '#AAF',
        fontSize: 30,
    },
    container: {
        display: 'flex',
        flexWrap: 'wrap',
        flexDirection: 'row',
        alignContent: 'center',
        justifyContent: 'space-around',
        padding: 10,
        borderColor: '#FFF',
        borderWidth: 3,
        borderRadius: 10,
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
        borderColor: '#00F',
        backgroundColor: '#222299aa',
        borderWidth: 3,
    },
    selectedBorder: {
        borderColor: '#00A',
        borderWidth: 5,
    },
    valueContainer: {
        display: 'flex',
        flexDirection: 'row',
        alignItems: 'center',
        gap: 10,
        borderColor: '#22991199',
        borderWidth: 3,
        backgroundColor: '#001155FF',
        padding: 5,
        borderRadius: 10,
    },
});
