import { Cross } from 'lucide-react-native';
import ThemedButton from './ui-elements/ThemedButton';

export interface CloseButtonProps {
    onPress: () => void;
}

export default function CloseButton({ onPress: callBack }: CloseButtonProps) {
    return (
        <ThemedButton
            icon={Cross}
            onPress={() => callBack()}
            style={{ padding: -5, opacity: 0.9, backgroundColor: '#FF000033' }}
            iconStyle={{
                transform: [{ rotate: '45deg' }],
                margin: -5,
                marginRight: -5,
            }}
            iconColor="red"
        ></ThemedButton>
    );
}
