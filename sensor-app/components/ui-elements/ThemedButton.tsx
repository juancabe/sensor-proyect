import React from 'react';
import { TouchableOpacity, StyleSheet, ViewStyle, TextStyle } from 'react-native';
import { LucideIcon } from 'lucide-react-native';
import { ThemedText } from './ThemedText';
import { useTheme } from '@react-navigation/native';

type ThemedButtonProps = {
    title?: string;
    icon?: LucideIcon;
    style?: ViewStyle;
    iconStyle?: ViewStyle;
    disabledStyle?: ViewStyle;
    textStyle?: TextStyle;
    onPress?: () => void;
    disabled?: boolean;
    iconColor?: string;
};

const ThemedButton: React.FC<ThemedButtonProps> = ({
    title,
    icon: Icon,
    style,
    iconStyle,
    disabledStyle,
    textStyle,
    onPress,
    disabled,
    iconColor,
}) => {
    const theme = useTheme();

    const color = iconColor ? iconColor : theme.colors.text;
    // const lightBg = 'yellow';
    const lightBg = '#4ed0ff';
    const darkBg = '#1e90ff';

    const customStyle = disabled ? disabledStyle : style;

    console.log('buttonStyle: ', style);

    return (
        <TouchableOpacity
            style={[
                styles.container,
                { backgroundColor: disabled ? 'grey' : theme.dark ? darkBg : lightBg },
                customStyle,
            ]}
            onPress={onPress}
            activeOpacity={0.7}
            disabled={disabled}
        >
            {Icon && (
                <Icon
                    size={20}
                    color={color}
                    style={[{ marginRight: title ? 8 : 0 }, iconStyle]}
                />
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
