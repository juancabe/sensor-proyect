import { StyleSheet, TouchableOpacity } from 'react-native';
import { ThemedView } from './ui-elements/ThemedView';

interface BindedColorPickerProps {
    selectedColor: string | undefined;
    onColorChange: (color: string) => void;
    colorValues: Record<string, string>;
}

export default function BindedColorPicker({
    selectedColor,
    onColorChange,
    colorValues,
}: BindedColorPickerProps) {
    return (
        <ThemedView style={styles.container}>
            {Object.entries(colorValues).map(([colorKey, hexValue]) => {
                const isSelected = selectedColor === colorKey;
                const isSelectedBorder = isSelected ? styles.selectedBorder : undefined;

                return (
                    <TouchableOpacity
                        key={colorKey}
                        style={[
                            { backgroundColor: hexValue },
                            styles.colorCircle,
                            isSelectedBorder,
                        ]}
                        onPress={() => onColorChange(colorKey)}
                    />
                );
            })}
        </ThemedView>
    );
}

const styles = StyleSheet.create({
    container: {
        display: 'flex',
        flexWrap: 'wrap',
        flexDirection: 'row',
        alignContent: 'center',
        justifyContent: 'space-between',
        padding: 10,
        borderColor: '#FFF',
        borderWidth: 3,
        borderRadius: 10,
        backgroundColor: '#000',
        gap: 6,
    },
    colorCircle: {
        width: 40,
        height: 40,
        borderRadius: 20,
    },
    selectedBorder: {
        borderColor: '#000',
        borderWidth: 5,
    },
});
