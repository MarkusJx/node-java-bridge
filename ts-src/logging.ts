import {
    setLogCallbacksInternal,
    initLogger as initLoggerInternal,
    resetLogCallbacks as resetLogCallbacksInternal,
    LOGGING_SUPPORTED as LOGGING_SUPPORTED_INTERNAL,
} from '../native';

/**
 * A namespace containing logging functions.
 *
 * The methods in this namespace are only available
 * if the native library was compiled with the `log` feature.
 * By default, the native library is compiled without this feature.
 *
 * If you don't know if the native library was compiled with the `log` feature,
 * the jsdoc comments of all methods in this namespace will start with
 * "This method is not supported in this build" if the native library
 * has been compiled without the `log` feature. Otherwise, the usual
 * jsdoc comments will be present. Also, the {@link logging.LOGGING_SUPPORTED}
 * constant can be used to check if the native library was compiled with
 * the `log` feature.
 *
 * ## Example
 * ```ts
 * import { logging } from 'java-bridge';
 *
 * logging.initLogger('log4rs.json');
 * logging.setLogCallbacks(
 *   (out) => console.log(out),
 *   (err) => console.error(err)
 * );
 * ```
 *
 * See {@link logging.initLogger} for further information
 * on how to configure the logger.
 *
 * @since 2.4.0
 */
export namespace logging {
    /**
     * A callback for logging.
     * This function must not throw any errors.
     * If an error is thrown, the application will exit.
     *
     * @param data the data to log
     */
    export type LogCallback = (data: string) => void;

    /**
     * @inheritDoc internal.initLogger
     */
    export function initLogger(path: string): void {
        return initLoggerInternal(path);
    }

    /**
     * @inheritDoc internal.setLogCallbacksInternal
     */
    export function setLogCallbacks(
        out?: LogCallback | null,
        error?: LogCallback | null
    ): void {
        const convertFunc = (func?: LogCallback | null) =>
            !!func
                ? (err?: object | null, data?: string | null) => {
                      if (err) {
                          // Err is (almost) always null
                          // Err may only be set if the string
                          // could not be converted to utf8.
                          // In this case, node.js will exit.
                          throw err;
                      }

                      func(data ?? '');
                  }
                : null;

        setLogCallbacksInternal(convertFunc(out), convertFunc(error));
    }

    /**
     * @inheritDoc internal.resetLogCallbacks
     */
    export function resetLogCallbacks(): void {
        return resetLogCallbacksInternal();
    }

    /**
     * @inheritDoc internal.LOGGING_SUPPORTED
     */
    export const LOGGING_SUPPORTED = LOGGING_SUPPORTED_INTERNAL;
}
