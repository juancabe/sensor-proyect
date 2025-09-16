import ThemedBackgroundView from '@/components/ui-elements/BackgroundView';
import PlacesScreen from '@/components/PlacesScreen';

const secondaryColor = '#ff00000a';

export default function Home() {
    return (
        <ThemedBackgroundView secondaryColor={secondaryColor}>
            <PlacesScreen />
        </ThemedBackgroundView>
    );
}
