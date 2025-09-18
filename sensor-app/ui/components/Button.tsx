import { Pressable, PressableProps, ViewStyle, StyleProp } from 'react-native';
import {
    createVariant,
    createRestyleComponent,
    VariantProps,
    useTheme,
} from '@shopify/restyle';
import { Text, type Theme } from '../theme';
import type { LucideIcon } from 'lucide-react-native';

// Base (buttonVariants must exist in your theme)
const ButtonBase = createRestyleComponent<
    VariantProps<Theme, 'buttonVariants'> &
        Omit<PressableProps, 'style'> & { style?: PressableProps['style'] },
    Theme
>([createVariant({ themeKey: 'buttonVariants' })], Pressable);

type ButtonProps = React.ComponentProps<typeof ButtonBase> & {
    label?: string;
    icon?: LucideIcon;
    iconPosition?: 'left' | 'right';
    iconSize?: number;
    iconColor?: string; // override if needed
    variant?: keyof Theme['buttonVariants'];
};

// helper: merge variant style (from Restyle) with user style (which may be a function)
function mergePressableStyle(
    variantStyle: StyleProp<ViewStyle> | undefined,
    userStyle?: PressableProps['style'],
): PressableProps['style'] {
    // Layout we want on buttons
    const rowCenter: ViewStyle = {
        flexDirection: 'row',
        alignItems: 'center',
        justifyContent: 'center',
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

    const finalStyle = mergePressableStyle(undefined, style);

    const hasLabel = !!label;
    const showLeftIcon = !!IconCmp && iconPosition === 'left';
    const showRightIcon = !!IconCmp && iconPosition === 'right';

    return (
        <ButtonBase
            variant={disabled ? 'disabled' : variant}
            accessibilityRole="button"
            accessibilityLabel={hasLabel ? label : accessibilityLabel}
            hitSlop={{ top: 6, bottom: 6, left: 6, right: 6 }}
            style={finalStyle}
            {...rest}
        >
            {showLeftIcon ? <IconCmp size={iconSize} color={resolvedIconColor} /> : null}
            {hasLabel ? (
                <Text
                    variant="body"
                    color={labelColorToken}
                    fontWeight="600"
                    marginLeft={showLeftIcon ? 's' : undefined}
                    marginRight={showRightIcon ? 's' : undefined}
                >
                    {label}
                </Text>
            ) : null}
            {showRightIcon ? <IconCmp size={iconSize} color={resolvedIconColor} /> : null}
        </ButtonBase>
    );
}
