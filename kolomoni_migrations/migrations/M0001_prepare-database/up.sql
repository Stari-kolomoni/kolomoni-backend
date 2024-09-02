CREATE
    ROLE kolomoni_migrator
    LOGIN
    PASSWORD 'kolomoni_migrator';

CREATE
    ROLE kolomoni_backend
    LOGIN
    PASSWORD 'kolomoni_backend';


REVOKE
    ALL PRIVILEGES
    ON DATABASE stari_kolomoni
    FROM PUBLIC;

GRANT
    ALL PRIVILEGES
    ON DATABASE stari_kolomoni
    TO kolomoni_migrator;

GRANT
    CONNECT, TEMPORARY
    ON DATABASE stari_kolomoni
    TO kolomoni_backend;



REVOKE
    ALL PRIVILEGES
    ON SCHEMA public
    FROM PUBLIC;



CREATE
    SCHEMA migrations
    AUTHORIZATION kolomoni_migrator;

GRANT
    ALL PRIVILEGES
    ON SCHEMA migrations
    TO kolomoni_migrator;



CREATE
    SCHEMA kolomoni
    AUTHORIZATION kolomoni_migrator;

GRANT
    ALL PRIVILEGES
    ON SCHEMA kolomoni
    TO kolomoni_migrator;

GRANT
    USAGE
    ON SCHEMA kolomoni
    TO kolomoni_backend;



ALTER DEFAULT PRIVILEGES
    FOR ROLE postgres
    IN SCHEMA migrations
    GRANT
        ALL PRIVILEGES
        ON TABLES
        TO kolomoni_migrator;


ALTER DEFAULT PRIVILEGES
    FOR ROLE kolomoni_migrator
    IN SCHEMA kolomoni
    GRANT
        SELECT, INSERT, UPDATE, DELETE
        ON TABLES
        TO kolomoni_backend;
