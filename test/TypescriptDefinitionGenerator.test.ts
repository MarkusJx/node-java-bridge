import { ModuleDeclaration, TypescriptDefinitionGenerator } from '../.';
import { expect } from 'chai';
import ts from 'typescript';
import path from 'path';
import * as fs from 'fs';
import isCi from 'is-ci';

interface Diagnostics {
    message: string;
    file: string;
    category: string;
    code: number;
}

let timeoutMs: number = 30e3;
if (isCi && (process.arch === 'arm64' || process.arch === 'arm')) {
    timeoutMs = 600e3;
}

function checkTypescriptSyntax(baseDirectory: string): Diagnostics[] {
    const program = ts.createProgram([path.join(baseDirectory, 'index.ts')], {
        checkJs: true,
        strict: true,
        target: ts.ScriptTarget.ES2022,
        module: ts.ModuleKind.CommonJS,
        moduleResolution: ts.ModuleResolutionKind.NodeJs,
        allowJs: true,
        noImplicitAny: true,
        noImplicitReturns: true,
        noImplicitThis: true,
        noUnusedLocals: true,
        noUnusedParameters: true,
        strictNullChecks: true,
        strictFunctionTypes: true,
        strictPropertyInitialization: true,
        noFallthroughCasesInSwitch: true,
        noImplicitOverride: true,
        esModuleInterop: true,
        resolveJsonModule: true,
        forceConsistentCasingInFileNames: true,
        allowSyntheticDefaultImports: true,
        strictBindCallApply: true,
        alwaysStrict: true,
    });

    const categoryToString = (category: ts.DiagnosticCategory): string => {
        switch (category) {
            case ts.DiagnosticCategory.Error:
                return 'error';
            case ts.DiagnosticCategory.Warning:
                return 'warning';
            case ts.DiagnosticCategory.Message:
                return 'message';
            case ts.DiagnosticCategory.Suggestion:
                return 'suggestion';
            default:
                return `unknown (${category})`;
        }
    };

    return ts.getPreEmitDiagnostics(program).map((d) => ({
        message: d.messageText as string,
        file: d.file?.fileName ?? 'unknown',
        category: categoryToString(d.category),
        code: d.code,
    }));
}

async function checkDeclarations(
    declarations: ModuleDeclaration[],
    indexContents: string
) {
    const dir = fs.mkdtempSync('java-bridge');
    const isSystemTest = path.basename(__dirname) === 'system_test';
    await TypescriptDefinitionGenerator.save(
        declarations.map((d) => ({
            name: d.name,
            contents: isSystemTest
                ? d.contents
                : d.contents.replaceAll(
                      'from "java-bridge";',
                      `from ${JSON.stringify(path.join(__dirname, '..'))};`
                  ),
        })),
        dir
    );

    await fs.promises.writeFile(
        path.join(dir, 'index.ts'),
        indexContents
            .trim()
            .split(/\r?\n/gi)
            .map((l) => l.trim())
            .join('\n')
    );

    const diagnostics = checkTypescriptSyntax(dir);
    await fs.promises.rm(dir, { recursive: true });

    expect(fs.existsSync(dir)).to.be.false;
    expect(diagnostics, JSON.stringify(diagnostics, null, 4)).to.be.empty;
}

describe('TypescriptDefinitionGenerator test', () => {
    it("Generate 'java.util.Iterator' definitions", async () => {
        const generator = new TypescriptDefinitionGenerator(
            'java.util.Iterator',
            null,
            []
        );

        const declarations = await generator.generate();
        expect(declarations.map((d) => d.name)).members([
            'java.util.Iterator',
            'java.util.function.Consumer',
        ]);
        expect(declarations.every((d) => d.contents.length > 0)).to.be.true;

        await checkDeclarations(
            declarations,
            `
            import { Iterator } from './java/util/Iterator';

            const iterator: Iterator | null = null;
            iterator!.instanceOf(Iterator);
            `
        );
    }).timeout(timeoutMs);
});
