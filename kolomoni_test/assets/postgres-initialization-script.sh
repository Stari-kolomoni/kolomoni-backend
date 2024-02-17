#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
	CREATE ROLE kolomoni WITH PASSWORD 'kolomoni' LOGIN;
	GRANT ALL ON DATABASE kolomoni TO kolomoni;
	ALTER DATABASE kolomoni OWNER TO kolomoni;
EOSQL
