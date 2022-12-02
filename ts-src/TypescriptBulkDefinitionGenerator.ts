import TypescriptDefinitionGenerator, {
    ModuleDeclaration,
    ProgressCallback,
} from './TypescriptDefinitionGenerator';

/**
 * Generates a lot of definitions for a lot of modules at once.
 * This caches the definitions for each module, so that if a module is used in multiple
 * other modules, it only needs to be generated once. This is a lot faster than generating
 * each module individually. It also generates the definitions in the correct order, so that
 * dependencies are always generated before the modules that depend on them.
 *
 * ## Example
 * ```ts
 * const generator = new TypescriptBulkDefinitionGenerator();
 *
 * // Generate definitions for the provided modules
 * await generator.generate([
 *      'java.lang.String',
 *      'java.util.List',
 *      'java.util.Map',
 *      'java.io.FileOutputStream',
 *      'java.io.FileInputStream',
 *      'java.io.File',
 *      'java.lang.System',
 * ]);
 *
 * // Save the definitions to a directory
 * await generator.save('javaDefinitions');
 * ```
 *
 * @see TypescriptDefinitionGenerator
 */
export class TypescriptBulkDefinitionGenerator {
    private readonly declarations: ModuleDeclaration[] = [];
    private readonly resolvedImports: string[] = [];

    /**
     * Generate the definitions for a module.
     *
     * @see TypescriptDefinitionGenerator.generate
     * @param classnames the fully qualified names of the classes to generate
     * @param progressCallback a callback that is called when a class is generated
     * @returns the number of classes that were generated
     */
    public async generate(
        classnames: string[],
        progressCallback: ProgressCallback | null = null
    ): Promise<number> {
        let numResolved = 0;
        for (const classname of classnames) {
            const generator = new TypescriptDefinitionGenerator(
                classname,
                progressCallback,
                this.resolvedImports
            );
            const generated = await generator.generate();
            numResolved += generated.length;

            this.declarations.push(...generated);
        }

        return numResolved;
    }

    /**
     * Save the generated definitions to a directory.
     *
     * @see TypescriptDefinitionGenerator.save
     * @param output the directory to save the definitions to
     */
    public async save(output: string): Promise<void> {
        await TypescriptDefinitionGenerator.save(this.declarations, output);
    }

    /**
     * Get the generated definitions.
     */
    public get moduleDeclarations(): ModuleDeclaration[] {
        return this.declarations;
    }
}
