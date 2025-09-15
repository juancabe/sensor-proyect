import { Modal, ModalProps, StyleSheet, View } from 'react-native';

export interface FeedbackModalProps extends ModalProps {}

export default function SensorsModal(props: FeedbackModalProps) {
    const { children, ...rest } = props;
    return (
        <Modal
            animationType={props.animationType ? props.animationType : 'slide'}
            transparent={false}
            backdropColor={'#0003'}
            {...rest}
        >
            <View style={[styles.container]}>{children}</View>
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
