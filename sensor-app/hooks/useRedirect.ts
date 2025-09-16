import { useRouter } from 'expo-router';

export default function useRedirect() {
    const router = useRouter();

    const redirectToIndex = () => {
        router.replace('/');
    };

    const redirectToLogin = () => {
        router.replace('/login');
    };

    return { redirectToIndex, redirectToLogin };
}
