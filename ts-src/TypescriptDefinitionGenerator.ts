import ts from 'typescript';
import { importClass } from './java';
import fs from 'fs';
import path from 'path';

const sourceFile = ts.createSourceFile('source.ts', '', ts.ScriptTarget.Latest, false, ts.ScriptKind.TS);

interface MethodDeclaration {
    returnType: string;
    parameters: string[];
    isStatic: boolean;
}

interface ModuleDeclaration {
    name: string;
    contents: string;
}

export default class TypescriptDefinitionGenerator {
    private usesBasicOrJavaType: boolean = false;
    private readonly additionalImports: string[] = [];
    private readonly importsToResolve: string[] = [];

    public constructor(private readonly classname: string, private readonly resolvedImports: string[] = []) {}

    private static convertMethods(methods: any[]): Record<string, MethodDeclaration[]> {
        const Modifier = importClass('java.lang.reflect.Modifier');

        const result: Record<string, MethodDeclaration[]> = {};
        for (const method of methods) {
            const modifiers = method.getModifiersSync();
            if (Modifier.isPublicSync(modifiers)) {
                const name = method.getNameSync();
                const data: MethodDeclaration = {
                    returnType: method.getReturnTypeSync().getTypeNameSync(),
                    parameters: method.getParameterTypesSync().map((p: any) => p.getTypeNameSync()),
                    isStatic: Modifier.isStaticSync(modifiers),
                };

                if (Object.hasOwn(result, name)) {
                    result[name].push(data);
                } else {
                    result[name] = [data];
                }
            }
        }

        return result;
    }

    private convertConstructors(constructors: any[]): ts.ClassElement[] {
        const Modifier = importClass('java.lang.reflect.Modifier');
        const types: string[][] = [];

        for (const constructor of constructors) {
            const modifiers = constructor.getModifiersSync();
            if (Modifier.isPublicSync(modifiers)) {
                types.push(constructor.getParameterTypesSync().map((p: any) => p.getTypeNameSync()));
            }
        }

        const tsConstructors = types.map((t, i) => {
            const params = t.map(this.convertParameter.bind(this));
            let declaration = ts.factory.createConstructorDeclaration(undefined, undefined, params, undefined);
            if (i === 0) {
                declaration = ts.addSyntheticLeadingComment(
                    declaration,
                    ts.SyntaxKind.SingleLineCommentTrivia,
                    ` ================== Constructors ==================`,
                    true
                );
            }

            if (t.length > 0) {
                declaration = ts.addSyntheticLeadingComment(
                    declaration,
                    ts.SyntaxKind.MultiLineCommentTrivia,
                    '*\n' + t.map((p, i) => ` * @param var${i} original type: '${p}'\n`).join('') + ' ',
                    true
                );
            }

            return declaration;
        });

        const newInstanceMethods = types.map((t, i) => {
            return this.createMethod(
                {
                    returnType: this.classname,
                    parameters: t,
                    isStatic: true,
                },
                'newInstance',
                i,
                false
            );
        });

        return [...newInstanceMethods, ...tsConstructors];
    }

    private javaTypeToTypescriptType(javaType: string): ts.TypeNode {
        if (javaType.endsWith('[]')) {
            return ts.factory.createArrayTypeNode(
                this.javaTypeToTypescriptType(javaType.substring(0, javaType.length - 2))
            );
        }

        switch (javaType) {
            case 'int':
            case 'java.lang.Integer':
            case 'long':
            case 'java.lang.Long':
            case 'float':
            case 'java.lang.Float':
            case 'double':
            case 'java.lang.Double':
            case 'byte':
            case 'java.lang.Byte':
            case 'short':
            case 'java.lang.Short':
                return ts.factory.createKeywordTypeNode(ts.SyntaxKind.NumberKeyword);
            case 'char':
            case 'java.lang.Character':
            case 'java.lang.String':
                return ts.factory.createKeywordTypeNode(ts.SyntaxKind.StringKeyword);
            case 'boolean':
            case 'java.lang.Boolean':
                return ts.factory.createKeywordTypeNode(ts.SyntaxKind.BooleanKeyword);
            case 'void':
            case 'java.lang.Void':
                return ts.factory.createKeywordTypeNode(ts.SyntaxKind.VoidKeyword);
            case 'java.lang.Object':
                this.usesBasicOrJavaType = true;
                return { ...ts.factory.createIdentifier('BasicOrJavaType'), _typeNodeBrand: '' };
            default:
                if (!this.resolvedImports.includes(javaType)) {
                    this.additionalImports.push(javaType);
                }

                this.importsToResolve.push(javaType);
                return {
                    ...ts.factory.createIdentifier(javaType.substring(javaType.lastIndexOf('.') + 1) + 'Class'),
                    _typeNodeBrand: '',
                };
        }
    }

    private convertParameter(param: string, index: number): ts.ParameterDeclaration {
        const name = 'var' + index;
        const type = this.javaTypeToTypescriptType(param);
        return ts.factory.createParameterDeclaration(undefined, undefined, undefined, name, undefined, type);
    }

    private convertParameters(params: MethodDeclaration) {
        return params.parameters.map(this.convertParameter.bind(this));
    }

    private static createMethodComment(declaration: MethodDeclaration) {
        return (
            '*\n' +
            declaration.parameters.map((p, i) => ` * @param var${i} original type: '${p}'\n`).join('') +
            ` * @return original return type: '${declaration.returnType}'\n `
        );
    }

    private createMethod(m: MethodDeclaration, name: string, i: number, isSync: boolean): ts.MethodDeclaration {
        const publicMod = ts.factory.createModifier(ts.SyntaxKind.PublicKeyword);
        const staticMod = ts.factory.createModifier(ts.SyntaxKind.StaticKeyword);

        const modifiers: ts.Modifier[] = [publicMod];
        if (m.isStatic) {
            modifiers.push(staticMod);
        }

        let returnType = this.javaTypeToTypescriptType(m.returnType);
        if (!isSync) {
            returnType = ts.factory.createTypeReferenceNode(ts.factory.createIdentifier('Promise'), [returnType]);
        }

        let declaration = ts.factory.createMethodDeclaration(
            undefined,
            modifiers,
            undefined,
            name + (isSync ? 'Sync' : ''),
            undefined,
            undefined,
            this.convertParameters(m),
            returnType,
            undefined
        );

        if (i === 0) {
            declaration = ts.addSyntheticLeadingComment(
                declaration,
                ts.SyntaxKind.SingleLineCommentTrivia,
                ` ================== Method ${name} ==================`,
                true
            );
        }

        return ts.addSyntheticLeadingComment(
            declaration,
            ts.SyntaxKind.MultiLineCommentTrivia,
            TypescriptDefinitionGenerator.createMethodComment(m),
            true
        );
    }

    private convertMethod(method: MethodDeclaration[], name: string): ts.MethodDeclaration[] {
        const res: ts.MethodDeclaration[] = [];

        for (let i = 0; i < method.length; i++) {
            const m = method[i];

            res.push(this.createMethod(m, name, i, false), this.createMethod(m, name, i, true));
        }

        return res;
    }

    private getAdditionalImports() {
        const getPath = (i: string) => {
            const thisSplit: (string | null)[] = this.classname.split('.');
            const importSplit: (string | null)[] = i.split('.');

            for (let j = 0; j < thisSplit.length; j++) {
                if (importSplit[j] === thisSplit[j]) {
                    thisSplit[j] = null;
                    importSplit[j] = null;
                } else {
                    break;
                }
            }

            return (
                './' +
                thisSplit
                    .filter((e) => !!e)
                    .map(() => '')
                    .join('../') +
                importSplit.filter((e) => !!e).join('/')
            );
        };

        const unique = <T>(value: T, index: number, self: T[]) => {
            return self.indexOf(value) === index;
        };

        return this.importsToResolve
            .filter((i) => i != this.classname)
            .filter(unique)
            .map((i) =>
                ts.factory.createImportDeclaration(
                    undefined,
                    undefined,
                    ts.factory.createImportClause(
                        false,
                        undefined,
                        ts.factory.createNamedImports([
                            ts.factory.createImportSpecifier(
                                false,
                                undefined,
                                ts.factory.createIdentifier(i.substring(i.lastIndexOf('.') + 1) + 'Class')
                            ),
                        ])
                    ),
                    ts.factory.createStringLiteral(getPath(i))
                )
            );
    }

    private getImports(): ts.ImportDeclaration {
        const importElements = [
            ts.factory.createImportSpecifier(false, undefined, ts.factory.createIdentifier('importClass')),
            ts.factory.createImportSpecifier(false, undefined, ts.factory.createIdentifier('JavaClassInstance')),
        ];

        if (this.usesBasicOrJavaType) {
            importElements.push(
                ts.factory.createImportSpecifier(false, undefined, ts.factory.createIdentifier('BasicOrJavaType'))
            );
        }

        const imports = ts.factory.createNamedImports(importElements);
        return ts.factory.createImportDeclaration(
            undefined,
            undefined,
            ts.factory.createImportClause(false, undefined, imports),
            ts.factory.createStringLiteral('@markusjx/java')
        );
    }

    private getExportStatement(simpleName: string) {
        const exportStatement = ts.factory.createVariableDeclaration(
            simpleName,
            undefined,
            undefined,
            ts.factory.createIdentifier(`importClass<typeof ${simpleName}Class>("${this.classname}")`)
        );

        return [
            ts.factory.createVariableStatement(undefined, [exportStatement]),
            ts.factory.createExportDefault(ts.factory.createIdentifier(simpleName)),
        ];
    }

    private getText(nodes: (ts.Node | null)[]) {
        return nodes
            .map(
                (n) =>
                    (n &&
                        ts
                            .createPrinter({ newLine: ts.NewLineKind.LineFeed })
                            .printNode(ts.EmitHint.Unspecified, n, sourceFile)) ||
                    ''
            )
            .join('\n');
    }

    public generate(): ModuleDeclaration[] {
        if (this.resolvedImports.includes(this.classname)) {
            return [];
        }

        this.resolvedImports.push(this.classname);
        console.log('Converting class', this.classname);

        const Class = importClass(this.classname);
        const cls = Class.class;

        const simpleName = cls.getNameSync().substring(cls.getNameSync().lastIndexOf('.') + 1);
        const methods = cls.getDeclaredMethodsSync();

        const classMembers: ts.ClassElement[] = [];

        const convertedMethods = TypescriptDefinitionGenerator.convertMethods(methods);
        for (const key of Object.keys(convertedMethods)) {
            const m = convertedMethods[key];
            classMembers.push(...this.convertMethod(m, key));
        }

        const constructors = cls.getDeclaredConstructorsSync();
        classMembers.push(...this.convertConstructors(constructors));

        const tsClass = ts.factory.createClassDeclaration(
            undefined,
            [
                ts.factory.createModifier(ts.SyntaxKind.ExportKeyword),
                ts.factory.createModifier(ts.SyntaxKind.DeclareKeyword),
            ],
            simpleName + 'Class',
            undefined,
            [
                ts.factory.createHeritageClause(ts.SyntaxKind.ExtendsKeyword, [
                    ts.factory.createExpressionWithTypeArguments(
                        ts.factory.createIdentifier('JavaClassInstance'),
                        undefined
                    ),
                ]),
            ],
            classMembers
        );

        const sourceText = this.getText([
            this.getImports(),
            ...this.getAdditionalImports(),
            null,
            tsClass,
            null,
            ...this.getExportStatement(simpleName),
        ]);

        const res: ModuleDeclaration[] = [];
        for (const imported of this.additionalImports) {
            res.push(...new TypescriptDefinitionGenerator(imported, this.resolvedImports).generate());
        }

        res.push({
            name: this.classname,
            contents: sourceText,
        });

        return res;
    }

    public static async save(declarations: ModuleDeclaration[], sourceDir: string): Promise<void> {
        for (const declaration of declarations) {
            const p = declaration.name.split('.');
            p[p.length - 1] = p[p.length - 1] + '.ts';

            const filePath = path.join(sourceDir, ...p);
            await fs.promises.mkdir(path.dirname(filePath), { recursive: true });
            await fs.promises.writeFile(filePath, declaration.contents, { encoding: 'utf8' });
        }
    }
}
