import { Card } from '@/ui/components/Card';
import { Text } from '@/ui/theme';

export interface ErrorBoxProps {
    error: string | null;
    style?: object;
}

export default function ErrorBox({ error, style }: ErrorBoxProps) {
    return (
        <Card variant="error" style={[style]} backgroundColor="bad">
            {error ? <Text variant="body">{error}</Text> : null}
        </Card>
    );
}
