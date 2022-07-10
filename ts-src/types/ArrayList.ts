import { importClass } from '../.';
import List from './List';

/**
 * A java array list.
 * Can be created using new.
 * Accepts all types List accepts.
 */
export default class ArrayList<T> extends importClass<typeof List>('java.util.ArrayList')<T> {}
