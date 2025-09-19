import { Card } from '@/ui/components/Card';
import { Theme } from '@/ui/theme';
import { useTheme } from '@shopify/restyle';
import { StyleSheet, TouchableOpacity } from 'react-native';

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
    const theme = useTheme<Theme>();
    const textColor = theme.colors.text;
    const bgColor = theme.colors.mainBackground;

    return (
        <Card
            variant="elevated"
            flexWrap="wrap"
            flexDirection="row"
            alignContent="center"
            justifyContent="space-between"
            width={290}
            style={{ backgroundColor: bgColor }}
            gap="m"
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
        </Card>
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
