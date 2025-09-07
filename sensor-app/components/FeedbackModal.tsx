import { Modal, ModalProps, StyleSheet, View } from 'react-native';

export interface FeedbackModalProps extends ModalProps {
    borderColor: string;
}

export default function FeedbackModal(props: FeedbackModalProps) {
    const { children, ...rest } = props;
    return (
        <Modal
            animationType={props.animationType ? props.animationType : 'slide'}
            transparent={false}
            backdropColor={'#0000'}
            {...rest}
        >
            <View style={[styles.container, { borderColor: props.borderColor }]}>
                {children}
            </View>
        </Modal>
    );
}

const styles = StyleSheet.create({
    container: {
        flex: 1,
        justifyContent: 'center',
        alignItems: 'center',
    },
});
