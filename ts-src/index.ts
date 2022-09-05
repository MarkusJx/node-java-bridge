export {
    JavaVersion,
    JavaObject,
    JavaClassInstance,
    JavaClassProxy,
    JavaType,
    JavaConstructor,
    BasicOrJavaType,
    BasicType,
    ImportedMembers,
    JavaClassType,
    Constructor,
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
export { getJavaLibPath } from '../native';
import TypescriptDefinitionGenerator from './TypescriptDefinitionGenerator';
export { TypescriptDefinitionGenerator };
export {
    ModuleDeclaration,
    MethodDeclaration,
    ProgressCallback,
} from './TypescriptDefinitionGenerator';
