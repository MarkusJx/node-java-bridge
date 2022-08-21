declare module 'nanobench' {
    export interface Benchmark {
        start(): void;

        end(): void;
    }

    export default function bench(
        name: string,
        func: (b: Benchmark) => void
    ): void;
}
