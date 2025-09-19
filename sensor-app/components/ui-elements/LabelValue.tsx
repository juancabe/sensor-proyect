import { Text } from '@/ui/theme';
import { BoxV } from '@/ui/components/BoxV';

// New props interface
export interface LabelValueProps {
    label: string;
    children: React.ReactNode; // Use children instead of values
    horizontal?: boolean;
}

export default function LabelValue(props: LabelValueProps) {
    return (
        <BoxV
            variant="field"
            backgroundColor="keyBackground"
            flexDirection={props.horizontal ? 'row' : 'column'}
            alignItems="center"
            style={{ padding: 0 }}
            justifyContent="space-between"
        >
            <Text variant="body" paddingRight="m" paddingLeft="m">
                {props.label}
            </Text>

            <BoxV variant="valueCell">{props.children}</BoxV>
        </BoxV>
    );
}
