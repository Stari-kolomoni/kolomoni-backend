DROP SCHEMA kolomoni CASCADE;
DROP SCHEMA migrations CASCADE;

GRANT
    ALL PRIVILEGES
    ON SCHEMA public
    TO PUBLIC;


REVOKE
    CONNECT, TEMPORARY
    ON DATABASE stari_kolomoni
    FROM kolomoni_backend;

REVOKE
    ALL PRIVILEGES
    ON DATABASE stari_kolomoni
    FROM kolomoni_migrator;

GRANT
    ALL PRIVILEGES
    ON DATABASE stari_kolomoni
    TO PUBLIC;


DROP ROLE kolomoni_backend;
DROP ROLE kolomoni_migrator;
