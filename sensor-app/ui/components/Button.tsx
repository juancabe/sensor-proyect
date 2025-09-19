import { Pressable, PressableProps, ViewStyle, StyleProp } from 'react-native';
import {
    createVariant,
    createRestyleComponent,
    VariantProps,
    useTheme,
} from '@shopify/restyle';
import { Box, Text, type Theme } from '../theme';
import type { LucideIcon } from 'lucide-react-native';
import { useEffect, useMemo, useState } from 'react';

// Base (buttonVariants must exist in your theme)
const ButtonBase = createRestyleComponent<
    VariantProps<Theme, 'buttonVariants'> &
        Omit<PressableProps, 'style'> & { style?: PressableProps['style'] },
    Theme
>([createVariant({ themeKey: 'buttonVariants' })], Pressable);

type ButtonProps = React.ComponentProps<typeof ButtonBase> & {
    label?: string;
    icon?: LucideIcon;
    iconPosition?: 'left' | 'right' | 'up' | 'down';
    iconSize?: number;
    iconColor?: string; // override if needed
    variant?: keyof Theme['buttonVariants'];
};

// helper: merge variant style (from Restyle) with user style (which may be a function)
function mergePressableStyle(
    variantStyle: StyleProp<ViewStyle> | undefined,
    userStyle?: PressableProps['style'],
    opacity?: number,
): PressableProps['style'] {
    // Layout we want on buttons
    const rowCenter: ViewStyle = {
        flexDirection: 'row',
        alignItems: 'center',
        justifyContent: 'center',
        opacity: opacity,
    };

    if (typeof userStyle === 'function') {
        return (state) => [variantStyle, rowCenter, userStyle(state)];
    }
    return [variantStyle, rowCenter, userStyle];
}

export function Button({
    label,
    icon: IconCmp,
    iconPosition = 'left',
    iconSize = 18,
    iconColor,
    variant = 'primary',
    style, // may be function or plain
    disabled,
    accessibilityLabel,
    ...rest
}: ButtonProps) {
    const theme = useTheme<Theme>();
    const labelColorToken = variant === 'ghost' ? 'clickable' : 'mainBackground';
    const resolvedIconColor = iconColor ?? theme.colors[labelColorToken];

    const hasLabel = !!label;
    const showLeftIcon = !!IconCmp && (iconPosition === 'left' || iconPosition === 'up');
    const showRightIcon =
        !!IconCmp && (iconPosition === 'right' || iconPosition === 'down');
    const vertical = iconPosition === 'up' || iconPosition === 'down';

    const [opacity, setOpacity] = useState<number | undefined>(undefined);

    const finalStyle = useMemo(() => {
        return mergePressableStyle(undefined, style, opacity);
    }, [opacity, style]);

    return (
        <ButtonBase
            variant={disabled ? 'disabled' : variant}
            accessibilityRole="button"
            accessibilityLabel={hasLabel ? label : accessibilityLabel}
            hitSlop={{ top: 6, bottom: 6, left: 6, right: 6 }}
            style={finalStyle}
            onHoverIn={() => setOpacity(0.8)}
            onHoverOut={() => setOpacity(undefined)}
            onPressIn={() => setOpacity(0.6)}
            onPressOut={() => setOpacity(undefined)}
            {...rest}
        >
            <Box
                flexDirection={vertical ? 'column' : 'row'}
                alignItems="center"
                justifyContent="space-between"
                gap="s"
            >
                {showLeftIcon ? (
                    <IconCmp size={iconSize} color={resolvedIconColor} />
                ) : null}
                {hasLabel ? (
                    <Text variant="body" color={labelColorToken} fontWeight="600">
                        {label}
                    </Text>
                ) : null}
                {showRightIcon ? (
                    <IconCmp size={iconSize} color={resolvedIconColor} />
                ) : null}
            </Box>
        </ButtonBase>
    );
}
