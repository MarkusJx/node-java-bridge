import {java_instance_proxy} from "./index";

export class CustomPersistence extends java_instance_proxy {
    public static createEntityManagerFactory(unitName: string, provider: java_instance_proxy): EntityManagerFactory;
}

export class EntityManagerFactory extends java_instance_proxy {
    public createEntityManager(): EntityManager;
}

export class EntityManager extends java_instance_proxy {
}

export class DatabaseManager extends java_instance_proxy {
    public constructor(entityManager: EntityManager);

    public close(): void;
}