import { StyleSheet, TouchableOpacity } from 'react-native';
import { ThemedView } from './ui-elements/ThemedView';
import { useTheme } from '@react-navigation/native';

interface BindedColorPickerProps {
    selectedColor: string | undefined;
    onColorChange: (color: string) => void;
    colorValues: string[];
}

export default function BindedColorPicker({
    selectedColor,
    onColorChange,
    colorValues,
}: BindedColorPickerProps) {
    const theme = useTheme();
    const bgColor = theme.colors.background;
    const textColor = theme.colors.text;

    return (
        <ThemedView
            style={[
                styles.container,
                { backgroundColor: bgColor, borderColor: selectedColor },
            ]}
        >
            {colorValues.map((color) => {
                const isSelected = selectedColor === color;
                const isSelectedBorder = isSelected ? styles.selectedBorder : undefined;

                return (
                    <TouchableOpacity
                        key={color}
                        style={[
                            { backgroundColor: color },
                            styles.colorCircle,
                            isSelectedBorder,
                            {
                                borderColor: textColor,
                            },
                        ]}
                        onPress={() => onColorChange(color)}
                    />
                );
            })}
        </ThemedView>
    );
}

const styles = StyleSheet.create({
    container: {
        width: 250,
        display: 'flex',
        flexWrap: 'wrap',
        flexDirection: 'row',
        alignContent: 'center',
        justifyContent: 'space-between',
        padding: 10,
        borderWidth: 3,
        borderRadius: 10,
        gap: 6,
    },
    colorCircle: {
        width: 40,
        height: 40,
        borderRadius: 20,
        borderWidth: 2,
    },
    selectedBorder: {
        borderColor: '#000',
        borderWidth: 5,
    },
});
