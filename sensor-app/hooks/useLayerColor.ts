import { rgbWithAlpha } from '@/helpers/withAlpha';
import { useTheme } from '@react-navigation/native';

export default function useLayerColor() {
    const theme = useTheme();
    return rgbWithAlpha(theme.colors.background, 0.33);
}
