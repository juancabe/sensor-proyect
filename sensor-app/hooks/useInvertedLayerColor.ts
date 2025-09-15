import { rgbWithAlpha } from '@/helpers/withAlpha';
import { useTheme } from '@react-navigation/native';

export default function useInvertedLayerColor(opacityMultiplier?: number) {
    const theme = useTheme();
    const baseOpacity = 0.33;
    const opacity = opacityMultiplier ? baseOpacity * opacityMultiplier : baseOpacity;
    return rgbWithAlpha(theme.colors.text, opacity);
}
