import {importClass, JavaType} from '../.';
import { List } from './List';

export declare class ArrayListClass<T extends JavaType> extends List<T> {
    public constructor(other: ArrayListClass<T>);
    public constructor();
}

/**
 * A java array list.
 * Can be created using new.
 * Accepts all types List accepts.
 */
export default class ArrayList<T> extends importClass<typeof ArrayListClass>('java.util.ArrayList')<T> {}
