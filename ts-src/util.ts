import { importClass, importClassAsync } from './java';

/**
 * Get the version of the Java VM in use.
 * Async version.
 *
 * @see getJavaVersionSync
 * @returns the java version string
 */
export async function getJavaVersion(): Promise<string> {
    const system = await importClassAsync('java.lang.System');
    return await system.getProperty('java.version');
}

/**
 * Get the version of the Java VM in use.
 * Sync version.
 *
 * This is equal to the following java implementation:
 * ```java
 * public static String getJavaVersion() {
 *     return System.getProperty("java.version");
 * }
 * ```
 *
 * @see getJavaVersion
 * @returns the java version string
 */
export function getJavaVersionSync(): string {
    const system = importClass('java.lang.System');
    return system.getPropertySync('java.version');
}
