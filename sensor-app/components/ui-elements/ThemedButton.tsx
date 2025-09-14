import React from 'react';
import { TouchableOpacity, StyleSheet, ViewStyle, TextStyle } from 'react-native';
import { LucideIcon } from 'lucide-react-native';
import { ThemedText } from './ThemedText';
import { useTheme } from '@react-navigation/native';

type ThemedButtonProps = {
    title?: string;
    icon?: LucideIcon;
    style?: ViewStyle;
    textStyle?: TextStyle;
    onPress?: () => void;
    disabled?: boolean;
};

const ThemedButton: React.FC<ThemedButtonProps> = ({
    title,
    icon: Icon,
    style,
    textStyle,
    onPress,
    disabled,
}) => {
    const theme = useTheme();

    const color = theme.colors.text;
    // const lightBg = 'yellow';
    const lightBg = '#4ed0ff';
    const darkBg = '#1e90ff';

    return (
        <TouchableOpacity
            style={[
                styles.container,
                style,
                { backgroundColor: disabled ? 'grey' : theme.dark ? darkBg : lightBg },
            ]}
            onPress={onPress}
            activeOpacity={0.8}
            disabled={disabled}
        >
            {Icon && (
                <Icon size={20} color={color} style={{ marginRight: title ? 8 : 0 }} />
            )}
            {title && <ThemedText style={[styles.text, textStyle]}>{title}</ThemedText>}
        </TouchableOpacity>
    );
};

const styles = StyleSheet.create({
    container: {
        flexDirection: 'row',
        alignItems: 'center',
        justifyContent: 'center',
        paddingVertical: 12,
        paddingHorizontal: 16,
        backgroundColor: '#1e90ff',
        borderRadius: 8,
    },
    text: {
        fontSize: 16,
        fontWeight: '600',
    },
});

export default ThemedButton;
