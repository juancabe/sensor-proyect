import { useRouter } from 'expo-router';

export default function useRedirect() {
    const router = useRouter();

    const redirectToIndex = () => {
        router.replace('/');
    };

    return { redirectToIndex };
}
