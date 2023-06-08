export {
    JavaVersion,
    JavaObject,
    JavaClassInstance,
    JavaClassProxy,
    JavaClass,
    JavaClassConstructor,
    UnknownJavaClass,
    JavaType,
    BasicOrJavaType,
    BasicType,
    JavaClassType,
    Constructor,
    UnknownJavaClassType,
    JavaClassConstructorType,
} from './definitions';
import type * as internal from '../native';
/**
 * A namespace containing any internal type definitions.
 * Do not actually use anything from this namespace
 * as it only exports types.
 */
export type { internal };
export * from './java';
import * as java from './java';
export default java;
export {
    getJavaLibPath,
    InterfaceProxyOptions,
    initLogger,
    resetLogCallbacks,
} from '../native';
export { getJavaVersion, getJavaVersionSync } from './util';
