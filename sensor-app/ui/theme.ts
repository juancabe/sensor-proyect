import { createBox, createText, createTheme } from '@shopify/restyle';

export interface ThemeColors {
    // Backgrounds
    mainBackground: string;
    cardBackground: string;
    fieldBackground: string;
    keyBackground: string;
    valueBackground: string;

    // Texts
    mainText: string;
    text: string;
    brandText: string;

    // Border
    borderOfCard: string;
    borderOfKey: string;
    borderOfValue: string;

    // Actions
    clickable: string;
    good: string;
    bad: string;
    warning: string;
    disabled: string;

    // Extra
    detail: string;
    info: string;
}

export const palette = {
    // Brand Colors
    brandRust: '#CC6B3E',
    brandGreen: '#2C9C6A',
    brandIndigo: '#3C4E6F',

    // Brand brightness variants
    brandRustLight: '#F0B9A1',
    brandRustDark: '#3C1F12',
    // exception
    brandRustReddish: '#CC433E',

    brandGreenLight: '#3CD791',
    brandGreenDark: '#185439',

    brandIndigoLight: '#6294F0',
    brandIndigoDark: '#2E4570',

    // Neutrals
    black: '#0B0B0B',
    charcoal: '#121417',

    grayLight: '#B8B8B8',
    gray: '#A1A1A1',
    grayDark: '#7D7D7D',

    panel: '#FAFBFC',
    white: '#F0F2F3',
    paper: '#FFFFFF',
};

export const lightColors: ThemeColors = {
    // Backgrounds
    mainBackground: palette.white, // app shell surface
    cardBackground: palette.paper, // keep cards neutral; use border/shadow for separation
    fieldBackground: palette.brandRustLight, // inputs readable with gray borders
    keyBackground: palette.grayLight, // subtle key column/chips backdrop
    valueBackground: palette.white, // keep values on clean surface

    // Texts
    mainText: palette.black, // body text for max readability
    text: palette.grayDark, // secondary text, metadata
    brandText: palette.brandRustDark,

    // Border
    borderOfCard: palette.grayLight, // light separation on white cards
    borderOfKey: palette.grayDark, // slightly stronger outline on gray key area
    borderOfValue: palette.grayLight, // soft outline on value side

    // Actions
    clickable: palette.brandIndigoLight, // primary links/buttons (brand Indigo)
    good: palette.brandGreen, // positive icons/accents; use brandGreenLight for bg tints
    bad: palette.brandRustReddish, // errors/destructive; reserve for true faults
    warning: palette.brandRust, // warnings/attention without full error
    disabled: palette.grayLight,

    // Extra
    detail: palette.brandRust,
    info: palette.brandRustDark,
};

export const darkColors: ThemeColors = {
    // Backgrounds
    mainBackground: palette.black, // app shell surface (very dark, low glare)
    cardBackground: palette.charcoal, // elevated panels/cards with subtle brand tint
    fieldBackground: palette.brandRustDark, // inputs slightly lighter than cards for affordance
    keyBackground: palette.brandIndigo, // key column/chips backdrop distinct from value
    valueBackground: palette.black, // values sit on clean, darkest surface

    // Texts
    mainText: palette.white, // primary text for max readability
    text: palette.gray, // secondary text, metadata on dark
    brandText: palette.brandRustLight,

    // Border
    borderOfCard: palette.brandIndigo, // soft, colored separation on dark cards
    borderOfKey: palette.brandIndigoLight, // clearer outline where keys need structure
    borderOfValue: palette.brandIndigo, // subtle outline on value area

    // Actions
    clickable: palette.brandIndigoLight, // primary links/buttons with strong contrast
    good: palette.brandGreenLight, // positive icons/accents visible on dark
    bad: palette.brandRustReddish, // destructive/error accents
    warning: palette.brandRust, // warnings/attention without harsh red
    disabled: palette.grayLight,

    // Extra
    detail: palette.brandRust,
    info: palette.brandRustLight,
};

const _makeRestyleTheme = (cs: 'dark' | 'light') => {
    const c = cs === 'dark' ? darkColors : lightColors;
    return createTheme({
        colors: {
            ...c,
        },

        spacing: {
            xs: 4,
            s: 8,
            m: 12,
            l: 16,
            xl: 24,
            xxl: 32,
        },
        borderRadii: {
            xs: 3,
            s: 6,
            m: 9,
            l: 12,
            xl: 19,
            xxl: 26,
            pill: 999,
        },

        textVariants: {
            body: { color: 'text', fontSize: 16, lineHeight: 22 },
            heading: {
                color: 'mainText',
                fontWeight: '600',
                fontSize: 18,
                lineHeight: 24,
            },
            title: {
                color: 'brandText',
                fontWeight: '700',
                fontSize: 36,
                lineHeight: 40,
                textShadowColor: 'text',
                textShadowOffset: { width: 1, height: 3 },
                textShadowRadius: 4,
            },
            subTitle: {
                color: 'brandText',
                fontWeight: '600',
                fontSize: 30,
                lineHeight: 34,
            },
            caption: { color: 'text', fontSize: 12, lineHeight: 16 },
            link: { color: 'clickable', fontWeight: '600' },
        },

        buttonVariants: {
            primary: {
                backgroundColor: 'clickable',
                borderRadius: 'm',
                paddingVertical: 's',
                paddingHorizontal: 'l',
            },
            ghost: {
                backgroundColor: 'mainBackground',
                borderColor: 'borderOfCard',
                borderWidth: 1,
                borderRadius: 'm',
                paddingVertical: 's',
                paddingHorizontal: 'l',
            },
            positive: {
                backgroundColor: 'good',
                borderRadius: 'm',
                paddingVertical: 's',
                paddingHorizontal: 'l',
            },
            negative: {
                backgroundColor: 'bad',
                borderRadius: 'm',
                paddingVertical: 's',
                paddingHorizontal: 'l',
            },
            warning: {
                backgroundColor: 'warning',
                borderRadius: 'm',
                paddingVertical: 's',
                paddingHorizontal: 'l',
            },
            disabled: {
                backgroundColor: 'disabled',
                borderRadius: 'm',
                paddingVertical: 's',
                marginHorizontal: 'm',
                paddingHorizontal: 'm',
            },
        },

        boxVariants: {
            field: {
                backgroundColor: 'fieldBackground',
                borderColor: 'borderOfCard',
                borderWidth: 1,
                borderRadius: 's',
                padding: 's',
            },
            keyCell: {
                borderColor: 'borderOfKey',
                backgroundColor: 'keyBackground',
                borderWidth: 1,
                borderRadius: 's',
                padding: 's',
            },
            valueCell: {
                borderColor: 'borderOfValue',
                backgroundColor: 'valueBackground',
                borderWidth: 1,
                borderRadius: 's',
                padding: 's',
            },
        },

        cardVariants: {
            elevated: {
                backgroundColor: 'cardBackground',
                borderColor: 'borderOfCard',
                borderWidth: 1,
                borderRadius: 'm',
                padding: 'm',
            },
            subtle: {
                backgroundColor: 'mainBackground',
                borderColor: 'borderOfCard',
                borderWidth: 1,
                borderRadius: 'm',
                padding: 'm',
            },
            error: {
                backgroundColor: 'cardBackground',
                borderColor: 'bad',
                borderWidth: 3,
                borderRadius: 'm',
                padding: 'm',
            },
        },
    });
};

const darkTheme = _makeRestyleTheme('dark');
const lightTheme = _makeRestyleTheme('light');

export function makeRestyleTheme(cs: 'dark' | 'light') {
    return cs === 'dark' ? darkTheme : lightTheme;
}

export type Theme = typeof darkTheme;

export const Box = createBox<Theme>();
export const Text = createText<Theme>();
