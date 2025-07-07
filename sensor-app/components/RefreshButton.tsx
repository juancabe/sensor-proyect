import { useEffect } from 'react';
import { StyleSheet } from 'react-native';
import Animated, {
  useAnimatedStyle,
  useSharedValue,
  withRepeat,
  withSequence,
  withTiming,
} from 'react-native-reanimated';

import { ThemedText } from '@/components/ThemedText';
import { IconSymbol } from './ui/IconSymbol';

import { useColorScheme } from '@/hooks/useColorScheme';
import { Colors } from '@/constants/Colors';

export function RefreshButton() {
  const rotationAnimation = useSharedValue(0);
  const colorScheme = useColorScheme();

  useEffect(() => {
    rotationAnimation.value = withRepeat(
      withSequence(
        withTiming(0, { duration: 0 }),
        withTiming(360, { duration: 500 })
      ),
      0 // run indefinitely
    );
  }, [rotationAnimation]);

  const animatedStyle = useAnimatedStyle(() => ({
    transform: [{ rotate: `${rotationAnimation.value}deg` }],
  }));

  return (
    <Animated.View style={animatedStyle}>
      <IconSymbol
        size={28}
        name="arrow.clockwise"
        color={Colors[colorScheme ?? 'light'].tint}
      />
    </Animated.View>
  );
}

const styles = StyleSheet.create({
  text: {
    fontSize: 28,
    lineHeight: 32,
    marginTop: -6,
  },
});
