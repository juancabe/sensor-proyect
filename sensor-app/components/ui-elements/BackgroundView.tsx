import React from 'react';
import { StyleSheet, useWindowDimensions, View } from 'react-native';
import { LinearGradient } from 'expo-linear-gradient';
import { BlurView } from 'expo-blur';
import { useTheme } from '@react-navigation/native';
import Svg, { Defs, RadialGradient, Stop, Circle, CircleProps } from 'react-native-svg';

type BackgroundViewProps = {
    children: React.ReactNode;
    secondaryColor?: string;
    style?: object;
};

export default function BackgroundView({
    children,
    secondaryColor,
    style,
}: BackgroundViewProps) {
    const theme = useTheme();
    const bg = theme.colors.background;
    const accent = secondaryColor ?? '#7c3aed';
    const { width, height } = useWindowDimensions();

    const aspectRatio = height / width;
    const mappedMaxHeight = aspectRatio * 100;

    const CIRCLES_NUM = 10;

    const circles: CircleProps[] = [];
    const maxRadious = 60;

    for (let i = 0; i < CIRCLES_NUM; i++) {
        const cx = Math.random() * 100;
        const cy = Math.random() * mappedMaxHeight;
        const r = Math.random() * maxRadious;
        circles.push({ cx, cy, r });
    }

    return (
        <View style={[styles.container, style]}>
            {/* Base wash */}
            <LinearGradient
                colors={[accent, bg]}
                start={{ x: 0, y: 0 }}
                end={{ x: 1, y: 1 }}
                style={StyleSheet.absoluteFill}
            />

            {/* Soft, fused aurora blobs */}

            <Svg
                width="100%"
                height="100%"
                viewBox={`0 0 100 ${mappedMaxHeight}`}
                style={[StyleSheet.absoluteFill]}
                pointerEvents="none"
            >
                <Defs>
                    {circles.map(({ cx, cy, r }, idx) => (
                        <RadialGradient
                            key={idx}
                            id={`g${idx}`}
                            cx={cx}
                            cy={cy}
                            r={r}
                            gradientUnits="userSpaceOnUse"
                        >
                            <Stop
                                offset="0"
                                stopColor={accent}
                                stopOpacity={`${r && (r.valueOf() as number) / maxRadious / 3}`}
                            />
                            <Stop offset="1" stopColor={accent} stopOpacity="0" />
                        </RadialGradient>
                    ))}
                </Defs>
                {circles.map(({ cx, cy, r }, idx) => {
                    const url = `url(#g${idx})`;
                    return <Circle key={idx} cx={cx} cy={cy} r={r} fill={url} />;
                })}
            </Svg>

            {/* Optional subtle blur to further blend shapes */}
            <BlurView
                intensity={20}
                style={StyleSheet.absoluteFill}
                pointerEvents="none"
            />

            <View style={styles.content}>{children}</View>
        </View>
    );
}

const styles = StyleSheet.create({
    container: { flex: 1 },
    content: { flex: 1, padding: 16 },
});
