import { MigrationInterface, QueryRunner } from "typeorm";

export class MigrationName1763102506297 implements MigrationInterface {
    name = 'MigrationName1763102506297'

    public async up(queryRunner: QueryRunner): Promise<void> {
        await queryRunner.query(`CREATE TYPE "public"."templates_type_enum" AS ENUM('email', 'push')`);
        await queryRunner.query(`CREATE TABLE "templates" ("id" uuid NOT NULL DEFAULT uuid_generate_v4(), "code" character varying NOT NULL, "type" "public"."templates_type_enum" NOT NULL, "language" character varying NOT NULL, "version" integer NOT NULL, "content" jsonb NOT NULL, "variables" jsonb NOT NULL, "created_at" TIMESTAMP NOT NULL DEFAULT now(), "updated_at" TIMESTAMP NOT NULL DEFAULT now(), CONSTRAINT "PK_515948649ce0bbbe391de702ae5" PRIMARY KEY ("id"))`);
    }

    public async down(queryRunner: QueryRunner): Promise<void> {
        await queryRunner.query(`DROP TABLE "templates"`);
        await queryRunner.query(`DROP TYPE "public"."templates_type_enum"`);
    }

}
