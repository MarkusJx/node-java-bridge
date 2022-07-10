import { importClass } from '../.';
import List from './List';

/**
 * A java array list.
 * Can be created using new.
 * Accepts all types List accepts.
 */
const ArrayList = importClass<typeof List>('java.util.ArrayList');
export default ArrayList;
