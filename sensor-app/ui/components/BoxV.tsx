import {
    createRestyleComponent,
    createVariant,
    VariantProps,
    backgroundColor,
    spacing,
    layout,
    border,
    shadow,
    type BackgroundColorProps,
    type SpacingProps,
    type LayoutProps,
    type BorderProps,
    type PositionProps,
    type ShadowProps,
} from '@shopify/restyle';
import { View } from 'react-native';
import type { Theme } from '../theme';

// Props supported by a Box + variant
type BoxBaseProps = BackgroundColorProps<Theme> &
    SpacingProps<Theme> &
    LayoutProps<Theme> &
    BorderProps<Theme> &
    PositionProps<Theme> &
    ShadowProps<Theme>;

export type BoxVariantProps = VariantProps<Theme, 'boxVariants'> &
    BoxBaseProps &
    React.ComponentProps<typeof View>;

// Important: put the variant FIRST, then the restyle functions.
// Later restyle props (like backgroundColor on the component) override the variant.
export const BoxV = createRestyleComponent<BoxVariantProps, Theme>(
    [
        createVariant({ themeKey: 'boxVariants' }),
        backgroundColor,
        spacing,
        layout,
        border,
        shadow,
    ],
    View,
);
