// Define PageLoad type locally since importing from ./$types causes issues
interface PageLoad {
    (): Promise<Record<string, any>>;
}

export const load = (async () => {
    return {};
}) satisfies PageLoad; 