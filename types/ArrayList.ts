import java from "../index";
import List from "./List";

/**
 * A java array list.
 * Can be created using new.
 * Accepts all types List accepts.
 */
const ArrayList = java.importClass('java.util.ArrayList') as typeof List;

export default ArrayList;
