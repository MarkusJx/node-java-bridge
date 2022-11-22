import TypescriptDefinitionGenerator, {
    ModuleDeclaration,
    ProgressCallback,
} from './TypescriptDefinitionGenerator';

export class TypescriptBulkDefinitionGenerator {
    private readonly declarations: ModuleDeclaration[] = [];
    private readonly resolvedImports: string[] = [];

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

    public async save(output: string): Promise<void> {
        await TypescriptDefinitionGenerator.save(this.declarations, output);
    }

    public get moduleDeclarations(): ModuleDeclaration[] {
        return this.declarations;
    }
}
